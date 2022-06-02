use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::remove_dir_all;
use test_log::test;
use testing::{examples, retry};

const TIMEOUT: &str = "60";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_assert() {
    fuzz("assert", false);
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true);
}

fn fuzz(krate: &str, persistent: bool) {
    let corpus = corpus_directory_from_target(krate, "target");

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(krate, "test").unwrap().assert().success();

    retry(3, || {
        let mut command = examples::test_fuzz(krate, "target").unwrap();

        let mut args = vec!["--exit-code", "--run-until-crash"];
        if persistent {
            args.push("--persistent");
        }
        args.extend_from_slice(&["--", "-V", TIMEOUT]);

        command.args(&args).assert().try_code(predicate::eq(1))
    })
    .unwrap();
}
