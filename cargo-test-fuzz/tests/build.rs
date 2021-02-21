use assert_cmd::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "../examples";

#[test]
fn build_no_instrumentation() {
    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--no-run", "--no-instrumentation"])
        .assert()
        .success();
}

#[test]
fn build() {
    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--no-run"])
        .assert()
        .success();
}

#[test]
fn build_pesistent() {
    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--no-run", "--persistent"])
        .assert()
        .success();
}
