use internal::{dirs::corpus_directory_from_target, fuzzer, serde_format, Fuzzer};
use predicates::prelude::*;
use std::{fs::remove_dir_all, sync::Mutex};
use testing::{examples, retry, skip_fuzzer, CommandExt};

const MAX_TOTAL_TIME: &str = "60";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_foo_qwerty() {
    // smoelius: When `bincode` is enabled, `cargo-afl` fails because "the program crashed with one
    // of the test cases provided."
    // smoelius: `to_string` is used here because `SerdeFormat` won't necessarily contain the
    // `Bincode` variant.
    let fuzzer = fuzzer().unwrap();
    fuzz(
        "test_foo_qwerty",
        if matches!(fuzzer, Fuzzer::Aflplusplus | Fuzzer::AflplusplusPersistent)
            && serde_format().to_string() == "Bincode"
        {
            2
        } else {
            1
        },
    );
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn fuzz_bar_asdfgh() {
    if ["Cbor", "Cbor4ii"].contains(&serde_format().to_string().as_str()) {
        skip_fuzzer!("fuzz_bar_asdfgh", Fuzzer::Libfuzzer);
    }

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

    examples::test("generic", test)
        .unwrap()
        .logged_assert()
        .success();

    retry(3, || {
        examples::test_fuzz("generic", "target")
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
