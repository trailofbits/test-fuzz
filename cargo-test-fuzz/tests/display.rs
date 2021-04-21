use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "../examples";

#[test]
fn display_qwerty() {
    display("qwerty", "Args { data: \"asdfgh\" }", "")
}

#[test]
fn display_debug_crash() {
    display(
        "debug_crash",
        "",
        "Encountered a failure while not replaying. A buggy Debug implementation perhaps?",
    )
}

#[test]
fn display_debug_hang() {
    display(
        "debug_hang",
        "",
        "Encountered a timeout while not replaying. A buggy Debug implementation perhaps?",
    )
}

fn display(name: &str, stdout: &str, stderr: &str) {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", name])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", name, "--display-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout))
        .stderr(predicate::str::contains(stderr));
}
