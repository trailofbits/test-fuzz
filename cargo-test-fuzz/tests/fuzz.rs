use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const TEST_DIR: &str = "../examples";

const TIMEOUT: &str = "60";

#[test]
fn fuzz_assert() {
    fuzz("assert", false)
}

#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true)
}

fn fuzz(target: &str, persistent: bool) {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", target])
        .assert()
        .success();

    let mut command = Command::new("cargo");

    let mut args = vec![
        "test-fuzz",
        "--target",
        target,
        "--no-ui",
        "--run-until-crash",
    ];
    if persistent {
        args.push("--persistent");
    }
    args.extend_from_slice(&["--", "-V", TIMEOUT]);

    command
        .current_dir(TEST_DIR)
        .args(&args)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("+++ Testing aborted programmatically +++")
                .and(predicate::str::contains("Time limit was reached").not()),
        );
}
