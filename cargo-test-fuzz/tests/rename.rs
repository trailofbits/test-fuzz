use assert_cmd::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "../examples";

#[test]
fn rename() {
    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--no-run", "--target", "rename"])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&[
            "test-fuzz",
            "--no-run",
            "--target",
            "rename",
            "--features",
            "bar_fuzz",
        ])
        .assert()
        .failure();
}
