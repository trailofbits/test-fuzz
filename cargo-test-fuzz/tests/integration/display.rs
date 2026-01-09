use predicates::prelude::*;
use testing::{LoggedAssert, fuzzable};

#[test]
fn display_qwerty() {
    display("qwerty", "test", "target", "Args { data: \"asdfgh\" }", "");
}

#[test]
fn display_debug_crash() {
    display(
        "debug",
        "crash::target_fuzz__::auto_generate",
        "crash::target",
        "",
        "Encountered a failure while not generating coverage and not replaying. A buggy Debug \
         implementation perhaps?",
    );
}

#[test]
fn display_debug_hang() {
    display(
        "debug",
        "hang::target_fuzz__::auto_generate",
        "hang::target",
        "",
        "Encountered a timeout while not generating coverage and not replaying. A buggy Debug \
         implementation perhaps?",
    );
}

fn display(krate: &str, test: &str, target: &str, stdout: &str, stderr: &str) {
    fuzzable::test(krate, test)
        .unwrap()
        .logged_assert()
        .success();

    fuzzable::test_fuzz(krate, target)
        .unwrap()
        .args(["--display=corpus"])
        .logged_assert()
        .success()
        .stdout(predicate::str::contains(stdout))
        .stderr(predicate::str::contains(stderr));
}
