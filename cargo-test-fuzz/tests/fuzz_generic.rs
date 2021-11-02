use internal::{dirs::corpus_directory_from_target, serde_format};
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::{fs::remove_dir_all, sync::Mutex};
use test_env_log::test;
use testing::{examples, retry};

const TIMEOUT: &str = "60";

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

#[test]
fn fuzz_bar_asdfgh() {
    fuzz("test_bar_asdfgh", 0);
}

#[cfg(test)]
lazy_static! {
    static ref LOCK: Mutex<()> = Mutex::new(());
}

fn fuzz(test: &str, code: i32) {
    let _guard = LOCK.lock().unwrap();

    let corpus = corpus_directory_from_target("generic", "target");

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test("generic", test).unwrap().assert().success();

    retry(3, || {
        examples::test_fuzz("generic", "target")
            .unwrap()
            .args(&["--exit-code", "--run-until-crash", "--", "-V", TIMEOUT])
            .assert()
            .try_code(predicate::eq(code))
    })
    .unwrap();
}
