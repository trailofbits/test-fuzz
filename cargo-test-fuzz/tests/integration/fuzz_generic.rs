use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::remove_dir_all;
use testing::{LoggedAssert, fuzzable, retry};

const MAX_TOTAL_TIME: &str = "60";

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_foo_qwerty() {
    // smoelius: Both `bincode` and `postcard` serialize enum variant indexes rather than their
    // names. This causes `cargo-afl` to fail because "the program crashed with one of the test
    // cases provided." Hence, the expected exit code is 2 (error) rather than 1 (crash found).
    fuzz("test_foo_qwerty", 2);
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_bar_asdfgh() {
    fuzz("test_bar_asdfgh", 0);
}

fn fuzz(test: &str, code: i32) {
    let corpus = corpus_directory_from_target("generic", "struct_target");

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(&corpus).unwrap_or_default();

    fuzzable::test("generic", test)
        .unwrap()
        .logged_assert()
        .success();

    assert!(corpus.exists());

    retry(3, || {
        fuzzable::test_fuzz("generic", "struct_target")
            .unwrap()
            .args([
                "--exit-code",
                "--run-until-crash",
                "--max-total-time",
                MAX_TOTAL_TIME,
            ])
            .logged_assert()
            .try_code(predicate::eq(code))
    })
    .unwrap();
}
