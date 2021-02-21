use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use std::{
    fs::{read_dir, remove_dir_all},
    process::Command,
};

const TEST_DIR: &str = "../examples";
const CRATE: &str = "assert";
const TARGET: &str = "assert::target";
const TIMEOUT: &str = "60";

#[test]
fn consolidate() {
    let corpus = corpus_directory_from_target(CRATE, TARGET);

    remove_dir_all(&corpus).unwrap_or_default();

    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", TARGET])
        .assert()
        .success();

    assert_eq!(read_dir(&corpus).unwrap().count(), 1);

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&[
            "test-fuzz",
            "--target",
            TARGET,
            "--no-ui",
            "--run-until-crash",
            "--",
            "-V",
            TIMEOUT,
        ])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", TARGET, "--consolidate"])
        .assert()
        .success();

    assert!(read_dir(&corpus).unwrap().count() > 1);
}
