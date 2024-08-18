use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};
use testing::{examples, CommandExt};

const MAX_TOTAL_TIME: &str = "60";

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn breadcrumbs() {
    const QWERTY: &str = "qwerty";

    let corpus = corpus_directory_from_target("qwerty_stepped", "target");

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test("qwerty_stepped", "test")
        .unwrap()
        .logged_assert()
        .success();

    assert_eq!(1, read_dir(&corpus).unwrap().count());

    for i in 1..=QWERTY.len() {
        let mut fuzz_flags = Vec::new();
        for j in i..QWERTY.len() {
            fuzz_flags.push(format!("--features=__qwerty_{}", &QWERTY[j..=j]));
        }

        // smoelius: A previous iteration could have found more than just one "qwerty" letter.
        if replay(&fuzz_flags) {
            continue;
        }

        examples::test_fuzz("qwerty_stepped", "target")
            .unwrap()
            .args([
                "--exit-code",
                "--run-until-crash",
                "--max-total-time",
                MAX_TOTAL_TIME,
            ])
            .args(&fuzz_flags)
            .logged_assert()
            .code(1);

        examples::test_fuzz("qwerty_stepped", "target")
            .unwrap()
            .args(["--consolidate"])
            .logged_assert()
            .success();

        assert!(replay(&fuzz_flags));
    }

    examples::test_fuzz("qwerty_stepped", "target")
        .unwrap()
        .args(["--display=corpus"])
        .logged_assert()
        .success()
        .stdout(predicate::str::contains(r#""qwerty""#));
}

fn replay(fuzz_flags: &[String]) -> bool {
    let assert = examples::test_fuzz("qwerty_stepped", "target")
        .unwrap()
        .args(["--replay=corpus"])
        .args(fuzz_flags)
        .logged_assert();

    let assert = assert.success();

    assert
        .try_stdout(predicate::str::contains(
            "thread 'target_fuzz::entry' panicked",
        ))
        .is_ok()
}
