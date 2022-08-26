use assert_cmd::Command;
use bitflags::bitflags;
use cargo_metadata::MetadataCommand;
use predicates::prelude::*;
use rustc_version::{version_meta, Channel};
use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
};
use tempfile::tempdir_in;
use test_log::test;

bitflags! {
    struct Flags: u8 {
        const EXPENSIVE = 0b0000_0001;
        const REQUIRES_ISOLATION = 0b0000_0010;
        const SKIP_NIGHTLY = 0b0000_0100;
    }
}

struct Test<'a> {
    flags: Flags,
    url: &'a str,
    patch: &'a str,
    subdir: &'a str,
    package: &'a str,
    targets: &'a [&'a str],
}

const TESTS: &[Test] = &[
    Test {
        flags: Flags::empty(),
        url: "https://github.com/CosmWasm/cw-plus",
        patch: "cw-plus.patch",
        subdir: ".",
        package: "cw20-base",
        targets: &["instantiate", "execute", "query", "migrate"],
    },
    Test {
        flags: Flags::EXPENSIVE,
        url: "https://github.com/paritytech/substrate",
        patch: "substrate_client_transaction_pool.patch",
        subdir: ".",
        package: "sc-transaction-pool",
        targets: &["import"],
    },
    // smoelius: Disable this test for now: https://github.com/solana-labs/solana/issues/25474
    // smoelius: It seems to be working again as of:
    // https://github.com/solana-labs/example-helloworld/pull/532
    Test {
        // smoelius: Keeping this here for reference.
        // https://github.com/bitflags/bitflags/issues/180#issuecomment-499302965
        // flags: Flags::from_bits_truncate(
        //     Flags::REQUIRES_ISOLATION.bits() | Flags::SKIP_NIGHTLY.bits(),
        // ),
        flags: Flags::REQUIRES_ISOLATION,
        url: "https://github.com/solana-labs/example-helloworld",
        patch: "example-helloworld.patch",
        subdir: "src/program-rust",
        package: "solana-bpf-helloworld",
        targets: &["process_instruction"],
    },
    Test {
        flags: Flags::EXPENSIVE,
        url: "https://github.com/solana-labs/solana",
        patch: "solana.patch",
        subdir: ".",
        package: "solana-bpf-loader-program",
        targets: &["process_instruction"],
    },
    Test {
        flags: Flags::empty(),
        url: "https://github.com/substrate-developer-hub/substrate-node-template",
        patch: "substrate_node_template.patch",
        subdir: ".",
        package: "pallet-template",
        targets: &["do_something"],
    },
];

// smoelius: This should match `scripts/update_patches.sh`.
const LINES_OF_CONTEXT: u32 = 2;

#[test]
fn cheap_tests() {
    let version_meta = version_meta().unwrap();
    for test in TESTS {
        run_test(
            test,
            test.flags.contains(Flags::EXPENSIVE)
                && (test.flags.contains(Flags::SKIP_NIGHTLY)
                    && version_meta.channel == Channel::Nightly),
        );
    }
}

#[test]
#[ignore]
fn all_tests() {
    let version_meta = version_meta().unwrap();
    for test in TESTS {
        run_test(
            test,
            test.flags.contains(Flags::SKIP_NIGHTLY) && version_meta.channel == Channel::Nightly,
        );
    }
}

fn run_test(test: &Test, stop_after_update: bool) {
    // smoelius: Each patch expects test-fuzz to be an ancestor of the directory in which the patch
    // is applied.
    #[allow(unknown_lints)]
    #[allow(env_cargo_path)]
    let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["clone", test.url, "."])
        .assert()
        .success();

    #[allow(unknown_lints)]
    #[allow(env_cargo_path)]
    let patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("patches")
        .join(test.patch)
        .canonicalize()
        .unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["apply", &patch.to_string_lossy()])
        .assert()
        .success();

    let subdir = tempdir.path().join(test.subdir);

    if test.flags.contains(Flags::REQUIRES_ISOLATION) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(subdir.join("Cargo.toml"))
            .unwrap();

        writeln!(file)
            .and_then(|_| writeln!(file, "[workspace]"))
            .unwrap();
    }

    // smoelius: Right now, Substrate's lockfile refers to `pin-project:0.4.27`, which is
    // incompatible with `syn:1.0.84`.
    Command::new("cargo")
        .current_dir(&subdir)
        .args(&["update"])
        .assert()
        .success();

    check_test_fuzz_dependency(&subdir, test.package);

    if stop_after_update {
        return;
    }

    Command::new("cargo")
        .current_dir(&subdir)
        .args(&["test", "--package", test.package, "--", "--nocapture"])
        .assert()
        .success();

    for target in test.targets {
        Command::cargo_bin("cargo-test-fuzz")
            .unwrap()
            .current_dir(&subdir)
            .args(&[
                "test-fuzz",
                "--package",
                test.package,
                "--display=corpus",
                target,
            ])
            .assert()
            .success()
            .stdout(predicate::str::is_match(r#"(?m)^[[:xdigit:]]{40}:"#).unwrap());

        Command::cargo_bin("cargo-test-fuzz")
            .unwrap()
            .current_dir(&subdir)
            .args(&[
                "test-fuzz",
                "--package",
                test.package,
                "--replay=corpus",
                target,
            ])
            .assert()
            .success()
            .stdout(
                predicate::str::is_match(r#"(?m)^[[:xdigit:]]{40}: Ret\((Ok|Err)\(.*\)\)$"#)
                    .unwrap(),
            );
    }
}

fn check_test_fuzz_dependency(subdir: &Path, test_package: &str) {
    let metadata = MetadataCommand::new()
        .current_dir(subdir)
        .no_deps()
        .exec()
        .unwrap();
    let package = metadata
        .packages
        .iter()
        .find(|package| package.name == test_package)
        .unwrap_or_else(|| panic!("Could not find package `{}`", test_package));
    let dep = package
        .dependencies
        .iter()
        .find(|dep| dep.name == "test-fuzz")
        .expect("Could not find dependency `test-fuzz`");
    assert!(dep.path.is_some());
}

#[test]
fn patches_are_current() {
    for test in TESTS {
        let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

        Command::new("git")
            .current_dir(tempdir.path())
            .args(&["clone", test.url, "."])
            .assert()
            .success();

        let patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("patches")
            .join(test.patch);
        let patch = read_to_string(patch_path).unwrap();

        Command::new("git")
            .current_dir(tempdir.path())
            .args(&["apply"])
            .write_stdin(patch.as_bytes())
            .assert()
            .success();

        let assert = Command::new("git")
            .current_dir(tempdir.path())
            .args(&["diff", &format!("--unified={}", LINES_OF_CONTEXT)])
            .assert()
            .success();

        let diff = String::from_utf8_lossy(&assert.get_output().stdout);

        assert_eq!(patch, diff);
    }
}
