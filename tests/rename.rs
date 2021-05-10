use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "examples";

#[test]
fn rename() {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--no-run"])
        .assert()
        .success();

    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--no-run", "--features", "bar_fuzz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the name `bar_fuzz` is defined multiple times",
        ));
}
