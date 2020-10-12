use assert_cmd::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "../examples";

#[test]
fn rename() {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--no-run", "--target", "rename"])
        .assert()
        .success();

    Command::new("cargo")
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
