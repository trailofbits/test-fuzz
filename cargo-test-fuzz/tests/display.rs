use predicates::prelude::*;
use test_log::test;
use testing::examples;

#[test]
fn display_qwerty() {
    display("qwerty", "test", "target", "Args { data: \"asdfgh\" }", "");
}

#[test]
fn display_debug_crash() {
    display(
        "debug",
        "crash::target_fuzz::auto_generate",
        "crash::target",
        "",
        "Encountered a failure while not replaying. A buggy Debug implementation perhaps?",
    );
}

#[test]
fn display_debug_hang() {
    display(
        "debug",
        "hang::target_fuzz::auto_generate",
        "hang::target",
        "",
        "Encountered a timeout while not replaying. A buggy Debug implementation perhaps?",
    );
}

fn display(krate: &str, test: &str, target: &str, stdout: &str, stderr: &str) {
    examples::test(krate, test).unwrap().assert().success();

    examples::test_fuzz(krate, target)
        .unwrap()
        .args(&["--display=corpus"])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout))
        .stderr(predicate::str::contains(stderr));
}
