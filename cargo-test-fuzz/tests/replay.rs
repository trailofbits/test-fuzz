use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use regex::Regex;
use rlimit::{Resource, Rlim};
use std::{fs::remove_dir_all, process::Command};

const TEST_DIR: &str = "../examples";

// smoelius: MEMORY_LIMIT must be large enough for the build process to complete.
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

const TIMEOUT: &str = "120";

enum What {
    Crashes,
    Hangs,
}

#[test]
fn replay_crashes() {
    replay(
        "alloc",
        "alloc::target",
        &[
            "--run-until-crash",
            "--",
            "-m",
            &format!("{}", MEMORY_LIMIT / 1024),
        ],
        &What::Crashes,
        &Regex::new(r"memory allocation of \d{10,} bytes failed\n").unwrap(),
    )
}

#[allow(clippy::trivial_regex)]
#[test]
fn replay_hangs() {
    replay(
        "parse_duration",
        "parse_duration::parse",
        &["--persistent", "--", "-V", TIMEOUT],
        &What::Hangs,
        &Regex::new(r"Timeout\n").unwrap(),
    )
}

fn replay(name: &str, target: &str, fuzz_args: &[&str], what: &What, re: &Regex) {
    let corpus = corpus_directory_from_target(name, target);

    remove_dir_all(&corpus).unwrap_or_default();

    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", name])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&["test-fuzz", "--target", target, "--reset"])
        .assert()
        .success();

    let mut args = vec!["test-fuzz", "--target", target, "--no-ui"];
    args.extend_from_slice(fuzz_args);

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(args)
        .assert()
        .success();

    // smoelius: The memory limit must be set to replay the crashes, but not the hangs.
    Resource::DATA
        .set(Rlim::from_raw(MEMORY_LIMIT), Rlim::from_raw(MEMORY_LIMIT))
        .unwrap();

    let mut command = Command::cargo_bin("cargo-test-fuzz").unwrap();

    command
        .current_dir(TEST_DIR)
        .args(&[
            "test-fuzz",
            "--target",
            target,
            match what {
                What::Crashes => "--replay-crashes",
                What::Hangs => "--replay-hangs",
            },
        ])
        .assert()
        .success();

    assert!(re.is_match(&String::from_utf8_lossy(
        &command.assert().get_output().stdout
    )));
}
