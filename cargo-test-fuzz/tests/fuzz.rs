use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::{fs::remove_dir_all, process::Command};

const TEST_DIR: &str = "../examples";

const TIMEOUT: &str = "60";

#[test]
fn fuzz_assert() {
    fuzz("assert", false)
}

#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true)
}

fn fuzz(name: &str, persistent: bool) {
    let corpus = corpus_directory_from_target(name, &format!("{}::target", name));

    remove_dir_all(&corpus).unwrap_or_default();

    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", name])
        .assert()
        .success();

    let mut command = Command::cargo_bin("cargo-test-fuzz").unwrap();

    let mut args = vec![
        "test-fuzz",
        "--target",
        name,
        "--no-ui",
        "--run-until-crash",
    ];
    if persistent {
        args.push("--persistent");
    }
    args.extend_from_slice(&["--", "-V", TIMEOUT]);

    command
        .current_dir(TEST_DIR)
        .args(&args)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("+++ Testing aborted programmatically +++")
                .and(predicate::str::contains("Time limit was reached").not()),
        );
}
