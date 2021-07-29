use internal::{dirs::corpus_directory_from_target, examples, serde_format};
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::{fs::remove_dir_all, sync::Mutex};

const NAME: &str = "generic";

const TARGET: &str = "generic::target";

const TIMEOUT: &str = "60";

#[test]
fn fuzz_foo_qwerty() {
    // smoelius: When `bincode` is enabled, `cargo-afl` fails because "the program crashed with one
    // of the test cases provided."
    if serde_format().to_string() == "Bincode" {
        fuzz("test_foo_qwerty", false, "", false);
    } else {
        fuzz(
            "test_foo_qwerty",
            true,
            "+++ Testing aborted programmatically +++",
            false,
        );
    };
}

#[test]
fn fuzz_bar_asdfgh() {
    fuzz(
        "test_bar_asdfgh",
        true,
        "+++ Testing aborted programmatically +++",
        true,
    );
}

#[cfg(test)]
lazy_static! {
    static ref LOCK: Mutex<()> = Mutex::new(());
}

fn fuzz(test: &str, success: bool, pattern: &str, timeout: bool) {
    let _guard = LOCK.lock().unwrap();

    let corpus = corpus_directory_from_target(NAME, TARGET);

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(NAME, &format!("{}::{}", NAME, test))
        .unwrap()
        .assert()
        .success();

    let assert = examples::test_fuzz(TARGET)
        .unwrap()
        .args(&["--no-ui", "--run-until-crash", "--", "-V", TIMEOUT])
        .assert();

    let assert = if success {
        assert.success()
    } else {
        assert.failure()
    };

    assert.stdout(
        predicate::str::contains(pattern).and(predicate::function(|s: &str| {
            if timeout {
                s.contains("Time limit was reached")
            } else {
                !s.contains("Time limit was reached")
            }
        })),
    );
}
