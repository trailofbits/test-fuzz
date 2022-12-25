use internal::{dirs::corpus_directory_from_target, Fuzzer};
use predicates::prelude::*;
use std::fs::remove_dir_all;
use testing::{examples, retry, skip_fuzzer, CommandExt};

const MAX_TOTAL_TIME: &str = "60";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_afraid_of_number_7() {
    fuzz("afraid_of_number_7");
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_assert() {
    fuzz("assert");
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_qwerty() {
    skip_fuzzer!("fuzz_qwerty", Fuzzer::Libfuzzer);

    fuzz("qwerty");
}

fn fuzz(krate: &str) {
    let corpus = corpus_directory_from_target(krate, "target");

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(corpus).unwrap_or_default();

    examples::test(krate, "test")
        .unwrap()
        .logged_assert()
        .success();

    retry(3, || {
        let mut command = examples::test_fuzz(krate, "target").unwrap();

        let mut args = vec!["--exit-code", "--run-until-crash"];
        args.extend_from_slice(&["--max-total-time", MAX_TOTAL_TIME]);

        command
            .args(&args)
            .logged_assert()
            .try_code(predicate::eq(1))
    })
    .unwrap();
}
