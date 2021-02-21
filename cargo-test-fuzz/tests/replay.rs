use assert_cmd::prelude::*;
use regex::Regex;
use rlimit::Resource;
use std::process::Command;

const TEST_DIR: &str = "../examples";

// smoelius: MEMORY_LIMIT must be large enough for the build process to complete.
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

#[test]
fn replay() {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", "alloc"])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", "alloc", "--reset"])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&[
            "test-fuzz",
            "--target",
            "alloc",
            "--no-ui",
            "--run-until-crash",
            "--",
            "-m",
            &format!("{}", MEMORY_LIMIT / 1024),
        ])
        .assert()
        .success();

    Resource::DATA.set(MEMORY_LIMIT, MEMORY_LIMIT).unwrap();

    let mut command = Command::cargo_bin("cargo-test-fuzz").unwrap();

    command
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", "alloc", "--replay-crashes"])
        .assert()
        .success();

    assert!(Regex::new(r"memory allocation of \d{10,} bytes failed\n")
        .unwrap()
        .is_match(&String::from_utf8_lossy(
            &command.assert().get_output().stdout
        )));
}
