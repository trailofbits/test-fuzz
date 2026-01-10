use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::remove_dir_all;
use testing::{CommandExt, fuzzable, retry};

const MAX_TOTAL_TIME: &str = "60";

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_assert() {
    fuzz("assert", false);
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true);
}

fn fuzz(krate: &str, persistent: bool) {
    let corpus = corpus_directory_from_target(krate, "target");

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(corpus).unwrap_or_default();

    fuzzable::test(krate, "test")
        .unwrap()
        .logged_assert()
        .success();

    retry(3, || {
        let mut command = fuzzable::test_fuzz(krate, "target").unwrap();

        let mut args = vec!["--exit-code", "--run-until-crash"];
        if persistent {
            args.push("--persistent");
        }
        args.extend_from_slice(&["--max-total-time", MAX_TOTAL_TIME]);

        command
            .args(args)
            .logged_assert()
            .try_code(predicate::eq(1))
    })
    .unwrap();
}
