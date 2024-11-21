use assert_cmd::assert::Assert;
use cargo_metadata::MetadataCommand;
use once_cell::sync::Lazy;
use option_set::option_set;
use predicates::prelude::*;
use rustc_version::{version_meta, Channel};
use serde::Deserialize;
use std::{
    ffi::OsStr,
    fs::read_to_string,
    io::{stderr, stdout, Write},
    path::Path,
    process::Command,
};
use tempfile::tempdir_in;
use testing::CommandExt;

option_set! {
    struct Flags: UpperSnake + u8 {
        const EXPENSIVE = 1 << 0;
        // const SKIP = 1 << 1;
        const SKIP_NIGHTLY = 1 << 2;
    }
}

#[derive(Deserialize)]
struct Test {
    flags: Flags,
    url: String,
    rev: String,
    patch: String,
    subdir: String,
    package: String,
    targets: Vec<String>,
}

static TESTS: Lazy<Vec<Test>> = Lazy::new(|| {
    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let content =
        read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("third_party.json")).unwrap();
    serde_json::from_str(&content).unwrap()
});

#[cfg_attr(dylint_lib = "supplementary", allow(commented_code))]
#[test]
fn test() {
    let version_meta = version_meta().unwrap();
    for test in TESTS.iter() {
        run_test(
            test,
            (!cfg!(feature = "test-third-party-full") && test.flags.contains(Flags::EXPENSIVE))
                    // || test.flags.contains(Flags::SKIP)
                    || (test.flags.contains(Flags::SKIP_NIGHTLY)
                        && version_meta.channel == Channel::Nightly),
        );
    }
}

#[allow(clippy::too_many_lines)]
fn run_test(test: &Test, no_run: bool) {
    #[allow(clippy::explicit_write)]
    writeln!(
        stderr(),
        "{}{}",
        test.url,
        if no_run { " (no-run)" } else { "" }
    )
    .unwrap();

    // smoelius: Each patch expects test-fuzz to be an ancestor of the directory in which the patch
    // is applied.
    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

    Command::new("git")
        .current_dir(&tempdir)
        .args(["clone", &test.url, "."])
        .logged_assert()
        .success();

    Command::new("git")
        .current_dir(&tempdir)
        .args(["checkout", &test.rev])
        .logged_assert()
        .success();

    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("patches")
        .join(&test.patch)
        .canonicalize()
        .unwrap();

    Command::new("git")
        .current_dir(&tempdir)
        .args(["apply", &patch.to_string_lossy()])
        .logged_assert()
        .success();

    let subdir = tempdir.path().join(&test.subdir);

    // smoelius: Right now, Substrate's lockfile refers to `pin-project:0.4.27`, which is
    // incompatible with `syn:1.0.84`.
    // smoelius: The `pin-project` issue has been resolved. But Substrate now chokes when
    // `libp2p-swarm-derive` is upgraded from 0.30.1 to 0.30.2.
    // smoelius: Substrate now chokes when `x25519-dalek` is upgraded from 2.0.0-pre.1 to 2.0.0.
    // However, rather than try to upgrade everything and then downgrade just `x25519-dalek`
    // (similar to how I did for `libp2p-swarm-derive`), I am instead trying to upgrade just the
    // packages that need it.
    // smoelius: `cw20-base` relies on `serde_json` 1.0.114, which is before `serde_json::Value`
    // started deriving `Hash`:
    // https://github.com/serde-rs/json/blob/e1b3a6d8a161ff5ec4865b487d148c17d0188e3e/src/value/mod.rs#L115
    for package in [
        "ahash@0.7.6",
        "ahash@0.7.7",
        "libc",
        "num-bigint@0.4.0",
        "serde_json",
        "tempfile",
    ] {
        #[allow(clippy::let_unit_value)]
        let () = Command::new("cargo")
            .current_dir(&subdir)
            .args(["update", "-p", package])
            .logged_assert()
            .try_success()
            .map_or((), |_| ());
    }

    // smoelius: The `libp2p-swarm-derive` issue appears to have been resolved.
    #[cfg(any())]
    {
        let mut command = Command::new("cargo");
        command
            .current_dir(&subdir)
            .args(["update", "-p", "libp2p-swarm-derive"]);
        if command.assert().try_success().is_ok() {
            command.args(["--precise", "0.30.1"]).assert().success();
        }
    }

    check_test_fuzz_dependency(&subdir, &test.package);

    if no_run {
        return;
    }

    // smoelius: Use `std::process::Command` so that we can see the output of the command as it
    // runs. `assert_cmd::Command` would capture the output.
    for flags in [&["--no-run", "--quiet"], &[]] as [&[&str]; 2] {
        assert!(Command::new("cargo")
            .current_dir(&subdir)
            .env_remove("RUSTUP_TOOLCHAIN")
            .env("TEST_FUZZ_WRITE", "1")
            .args([
                "test",
                "--package",
                &test.package,
                "--features",
                &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
            ])
            .args(flags)
            .args(["--", "--nocapture"])
            .status()
            .unwrap()
            .success());
    }

    for target in &test.targets {
        test_fuzz(&subdir, &test.package, target, ["--display=corpus"])
            .stdout(predicate::str::is_match(r"(?m)^[[:xdigit:]]{40}:").unwrap())
            .stdout(predicate::str::contains("unknown of type").not());

        test_fuzz(&subdir, &test.package, target, ["--replay=corpus"]).stdout(
            predicate::str::is_match(r"(?m)^[[:xdigit:]]{40}: Ret\((Ok|Err)\(.*\)\)$").unwrap(),
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
        .unwrap_or_else(|| panic!("Could not find package `{test_package}`"));
    let dep = package
        .dependencies
        .iter()
        .find(|dep| dep.name == "test-fuzz")
        .expect("Could not find dependency `test-fuzz`");
    assert!(dep.path.is_some());
}

fn test_fuzz<P, I, S>(subdir: P, package: &str, target: &str, args: I) -> Assert
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("cargo")
        .args(["build", "--package=cargo-test-fuzz"])
        .logged_assert()
        .success();

    let metadata = MetadataCommand::new().no_deps().exec().unwrap();

    let assert = Command::new(metadata.target_directory.join("debug/cargo-test-fuzz"))
        .current_dir(subdir)
        .env("RUST_LOG", "debug")
        .args([
            "test-fuzz",
            "--package",
            package,
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
            target,
        ])
        .args(args)
        .logged_assert()
        .success();
    stderr().write_all(&assert.get_output().stderr).unwrap();
    stdout().write_all(&assert.get_output().stdout).unwrap();
    assert
}

#[cfg(feature = "test-third-party-full")]
#[test]
fn patches_are_current() {
    // smoelius: This should match `scripts/update_patches.sh`.
    const LINES_OF_CONTEXT: u32 = 2;

    let re = regex::Regex::new(r"^index [[:xdigit:]]{7}\.\.[[:xdigit:]]{7} [0-7]{6}$").unwrap();

    for test in TESTS.iter() {
        let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

        Command::new("git")
            .current_dir(&tempdir)
            .args(["clone", "--depth=1", &test.url, "."])
            .logged_assert()
            .success();

        let patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("patches")
            .join(&test.patch);
        let patch = read_to_string(patch_path).unwrap();

        // smoelius: To use `std::process::Command` here, the `write_stdin` would have to be
        // removed.
        #[allow(clippy::disallowed_methods)]
        assert_cmd::Command::new("git")
            .current_dir(&tempdir)
            .args(["apply"])
            .write_stdin(patch.as_bytes())
            .assert()
            .success();

        // smoelius: The following checks are *not* redundant. They can fail even if the patch
        // applies.

        // smoelius: Don't use `logged_assert` here. It now calls `strip_ansi_escapes::strip` which
        // escapes certain whitespace characters and mess up the diff.
        #[allow(clippy::disallowed_methods)]
        let assert = assert_cmd::Command::new("git")
            .current_dir(&tempdir)
            .args(["diff", &format!("--unified={LINES_OF_CONTEXT}")])
            .assert()
            .success();

        let diff = std::str::from_utf8(&assert.get_output().stdout).unwrap();

        let patch_lines = patch.lines().collect::<Vec<_>>();
        let diff_lines = diff.lines().collect::<Vec<_>>();

        assert_eq!(patch_lines.len(), diff_lines.len());

        for (patch_line, diff_line) in patch_lines.into_iter().zip(diff_lines) {
            if !(re.is_match(patch_line) && re.is_match(diff_line)) {
                assert_eq!(patch_line, diff_line);
            }
        }
    }
}

// smoelius: I am disabling this test, as it creates a race to get the patches committed after they
// are updated.
//   Note that `patches_are_current` above remains, and that it tests each patch against the head of
// its respective repository, not the revision against which the patch was generated.
#[cfg(any())]
#[test]
#[ignore]
fn revisions_are_current() {
    for test in TESTS.iter() {
        let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

        Command::new("git")
            .current_dir(&tempdir)
            .args(["clone", "--depth=1", &test.url, "."])
            .logged_assert()
            .success();

        let assert = Command::new("git")
            .current_dir(&tempdir)
            .args(["rev-parse", "HEAD"])
            .logged_assert()
            .success();

        let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap();

        assert_eq!(stdout.trim_end(), test.rev);
    }
}
