use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::remove_dir_all;
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

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn display_multiple_corpora() {
    for i in 0..6 {
        let target = format!("target_{i}");
        let corpus = corpus_directory_from_target("parallel", &target);
        remove_dir_all(corpus).unwrap_or_default();
    }

    fuzzable::test("parallel", "test")
        .unwrap()
        .logged_assert()
        .success();

    let mut assert = fuzzable::test_fuzz_inexact("parallel", "target")
        .unwrap()
        .args(["--display=corpus"])
        .logged_assert()
        .success();

    for i in 0..6 {
        assert = assert
            .try_stdout(predicate::str::is_match(format!(r"(?m)^target_{i} -+$")).unwrap())
            .unwrap();
    }
}
