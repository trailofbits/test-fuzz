use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::{fs::remove_dir_all, process::Command, sync::Mutex};

const NAME: &str = "generic";

const TARGET: &str = "generic::target";

const TEST_DIR: &str = "../examples";

const TIMEOUT: &str = "60";

#[test]
fn fuzz_foo_qwerty() {
    fuzz("test_foo_qwerty", false)
}

#[test]
fn fuzz_bar_asdfgh() {
    fuzz("test_bar_asdfgh", true)
}

#[cfg(test)]
lazy_static! {
    static ref LOCK: Mutex<()> = Mutex::new(());
}

fn fuzz(test: &str, timeout: bool) {
    let _guard = LOCK.lock().unwrap();

    let corpus = corpus_directory_from_target(NAME, TARGET);

    remove_dir_all(&corpus).unwrap_or_default();

    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&[
            "test",
            "--",
            "--exact",
            "--test",
            &format!("{}::{}", NAME, test),
        ])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(TEST_DIR)
        .args(&[
            "test-fuzz",
            "--exact",
            "--target",
            TARGET,
            "--no-ui",
            "--run-until-crash",
            "--",
            "-V",
            TIMEOUT,
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("+++ Testing aborted programmatically +++").and(
                predicate::function(|s: &str| {
                    if timeout {
                        s.contains("Time limit was reached")
                    } else {
                        !s.contains("Time limit was reached")
                    }
                }),
            ),
        );
}
