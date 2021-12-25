use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use testing::examples::MANIFEST_PATH;

#[test]
fn rename() {
    Command::new("cargo")
        .args(&[
            "test",
            "--manifest-path",
            &MANIFEST_PATH,
            "--no-run",
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format().as_feature()),
        ])
        .assert()
        .success();

    Command::new("cargo")
        .args(&[
            "test",
            "--manifest-path",
            &MANIFEST_PATH,
            "--no-run",
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format().as_feature()),
            "--features",
            "__bar_fuzz",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the name `bar_fuzz` is defined multiple times",
        ));
}
