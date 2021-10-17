use predicates::prelude::*;
use testing::examples;

#[test]
fn display_qwerty() {
    display(
        "qwerty",
        "qwerty::test",
        "qwerty::target",
        "Args { data: \"asdfgh\" }",
        "",
    );
}

#[test]
fn display_debug_crash() {
    display(
        "debug",
        "debug_crash::target_fuzz::auto_generate",
        "debug_crash::target",
        "",
        "Encountered a failure while not replaying. A buggy Debug implementation perhaps?",
    );
}

#[test]
fn display_debug_hang() {
    display(
        "debug",
        "debug_hang::target_fuzz::auto_generate",
        "debug_hang::target",
        "",
        "Encountered a timeout while not replaying. A buggy Debug implementation perhaps?",
    );
}

fn display(name: &str, test: &str, target: &str, stdout: &str, stderr: &str) {
    examples::test(name, test).unwrap().assert().success();

    examples::test_fuzz(name, target)
        .unwrap()
        .args(&["--display-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::contains(stdout))
        .stderr(predicate::str::contains(stderr));
}
