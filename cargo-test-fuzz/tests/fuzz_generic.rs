use internal::{dirs::corpus_directory_from_target, serde_format};
use predicates::prelude::*;
use std::{fs::remove_dir_all, sync::Mutex};
use test_log::test;
use testing::{examples, retry};

const TIMEOUT: &str = "60";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_foo_qwerty() {
    // smoelius: When `bincode` is enabled, `cargo-afl` fails because "the program crashed with one
    // of the test cases provided."
    if serde_format().to_string() == "Bincode" {
        fuzz("test_foo_qwerty", 2);
    } else {
        fuzz("test_foo_qwerty", 1);
    };
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_bar_asdfgh() {
    fuzz("test_bar_asdfgh", 0);
}

static MUTEX: Mutex<()> = Mutex::new(());

fn fuzz(test: &str, code: i32) {
    let _lock = MUTEX.lock().unwrap();

    let corpus = corpus_directory_from_target("generic", "target");

    // smoelius: This call to `remove_dir_all` is protected by the mutex above.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(corpus).unwrap_or_default();

    examples::test("generic", test).unwrap().assert().success();

    retry(3, || {
        examples::test_fuzz("generic", "target")
            .unwrap()
            .args(["--exit-code", "--run-until-crash", "--", "-V", TIMEOUT])
            .assert()
            .try_code(predicate::eq(code))
    })
    .unwrap();
}
