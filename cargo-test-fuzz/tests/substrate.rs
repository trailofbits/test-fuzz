use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs::read_to_string, path::Path};
use tempfile::tempdir_in;
use test_log::test;

struct Test<'a> {
    expensive: bool,
    url: &'a str,
    patch: &'a str,
    package: &'a str,
}

const TESTS: &[Test] = &[
    Test {
        expensive: true,
        url: "https://github.com/paritytech/substrate",
        patch: "substrate_client_transaction_pool.patch",
        package: "sc-transaction-pool",
    },
    Test {
        expensive: false,
        url: "https://github.com/substrate-developer-hub/substrate-node-template",
        patch: "substrate_node_template.patch",
        package: "pallet-template",
    },
];

// smoelius: This should match `scripts/update_substrate_patches.sh`.
const LINES_OF_CONTEXT: u32 = 2;

#[test]
fn cheap_tests() {
    TESTS
        .iter()
        .filter(|test| !test.expensive)
        .for_each(run_test);
}

#[test]
#[ignore]
fn expensive_tests() {
    TESTS
        .iter()
        .filter(|test| test.expensive)
        .for_each(run_test);
}

fn run_test(test: &Test) {
    // smoelius: Each patch expects test-fuzz to be an ancestor of the directory in which the patch
    // is applied.
    let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["clone", test.url, "."])
        .assert()
        .success();

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

    Command::new("cargo")
        .current_dir(tempdir.path())
        .args(&["test", "--package", test.package, "--", "--nocapture"])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(tempdir.path())
        .args(&["test-fuzz", "--package", test.package, "--display-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"(?m)^[[:xdigit:]]{40}:"#).unwrap());

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(tempdir.path())
        .args(&["test-fuzz", "--package", test.package, "--replay-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"(?m)^[[:xdigit:]]{40}: Ret\(Ok\(.*\)\)$"#).unwrap());
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
