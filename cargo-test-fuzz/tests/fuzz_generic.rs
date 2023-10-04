use internal::{dirs::corpus_directory_from_target, serde_format};
use predicates::prelude::*;
use std::{fs::remove_dir_all, sync::Mutex};
use testing::{examples, retry, CommandExt};

const TIMEOUT: &str = "60";

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_foo_qwerty() {
    // smoelius: When `bincode` is enabled, `cargo-afl` fails because "the program crashed with one
    // of the test cases provided."
    fuzz(
        "test_foo_qwerty",
        if serde_format::serializes_variant_names() {
            1
        } else {
            2
        },
    );
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_bar_asdfgh() {
    fuzz("test_bar_asdfgh", 0);
}

static MUTEX: Mutex<()> = Mutex::new(());

fn fuzz(test: &str, code: i32) {
    let _lock = MUTEX.lock().unwrap();

    let corpus = corpus_directory_from_target("generic", "target");

    // smoelius: This call to `remove_dir_all` is protected by the mutex above.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(corpus).unwrap_or_default();

    examples::test("generic", test)
        .unwrap()
        .logged_assert()
        .success();

    retry(3, || {
        examples::test_fuzz("generic", "target")
            .unwrap()
            .args(["--exit-code", "--run-until-crash", "--", "-V", TIMEOUT])
            .logged_assert()
            .try_code(predicate::eq(code))
    })
    .unwrap();
}
