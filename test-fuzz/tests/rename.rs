use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::{path::Path, process::Command};

lazy_static! {
    static ref MANIFEST_PATH: String = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("Cargo.toml")
        .to_string_lossy()
        .to_string();
}

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
