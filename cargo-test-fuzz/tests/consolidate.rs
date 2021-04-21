use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use std::{
    fs::{read_dir, remove_dir_all},
    process::Command,
};

const TEST_DIR: &str = "../examples";

const CRASH_TIMEOUT: &str = "60";

const HANG_TIMEOUT: &str = "120";

#[test]
fn consolidate_crashes() {
    consolidate(
        "assert",
        "assert::target",
        &["--run-until-crash", "--", "-V", CRASH_TIMEOUT],
    );
}

#[test]
fn consolidate_hangs() {
    consolidate(
        "parse_duration",
        "parse_duration::parse",
        &["--persistent", "--", "-V", HANG_TIMEOUT],
    );
}

fn consolidate(name: &str, target: &str, fuzz_args: &[&str]) {
    let corpus = corpus_directory_from_target(name, target);

    remove_dir_all(&corpus).unwrap_or_default();

    // smoelius: Do not run additional tests (e.g., the default tests), so that at most one corpus
    // entry is produced.
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&[
            "test",
            "--",
            "--exact",
            "--test",
            &format!("{}::test", name),
        ])
        .assert()
        .success();

    assert_eq!(read_dir(&corpus).unwrap().count(), 1);

    let mut args = vec!["test-fuzz", "--target", target, "--no-ui"];
    args.extend_from_slice(fuzz_args);

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(args)
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", target, "--consolidate"])
        .assert()
        .success();

    assert!(read_dir(&corpus).unwrap().count() > 1);
}
