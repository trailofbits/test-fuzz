use internal::dirs::{corpus_directory_from_target, output_directory_from_target};
use predicates::prelude::*;
use std::{ffi::OsStr, fs::remove_dir_all};
use testing::{examples, retry, CommandExt};

const CPUS: &str = "2";
const TIME_SLICE: &str = "30";

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn fuzz_parallel() {
    for i in 0..6 {
        let output_dir = output_directory_from_target("parallel", &format!("target_{i}"));
        remove_dir_all(output_dir).unwrap_or_default();
    }

    examples::test("parallel", "test")
        .unwrap()
        .logged_assert()
        .success();

    retry(3, || {
        examples::test_fuzz_inexact("parallel", "target")
            .unwrap()
            .args([
                "--exit-code",
                "--run-until-crash",
                "--cpus",
                CPUS,
                "--slice",
                TIME_SLICE,
            ])
            .logged_assert()
            .try_code(predicate::eq(1))
    })
    .unwrap();

    // smoelius: Verify that all `.cur_input` files were removed.
    for i in 0..6 {
        let output_dir = output_directory_from_target("parallel", &format!("target_{i}"));
        if output_dir.exists() {
            assert!(!walkdir::WalkDir::new(output_dir)
                .into_iter()
                .any(|entry| entry.unwrap().path().file_name() == Some(OsStr::new(".cur_input"))));
        }
    }
}

const MAX_TOTAL_TIME: &str = "10";

#[test]
fn no_premature_termination() {
    let corpus_calm = corpus_directory_from_target("calm_and_panicky", "calm");
    remove_dir_all(&corpus_calm).unwrap_or_default();

    let corpus_panicky = corpus_directory_from_target("calm_and_panicky", "panicky");
    remove_dir_all(&corpus_panicky).unwrap_or_default();

    examples::test("calm_and_panicky", "test")
        .unwrap()
        .logged_assert()
        .success();

    let assert = retry(3, || {
        examples::test_fuzz_inexact("calm_and_panicky", "")
            .unwrap()
            .args([
                "--exit-code",
                "--run-until-crash",
                "--cpus",
                CPUS,
                "--max-total-time",
                MAX_TOTAL_TIME,
            ])
            .logged_assert()
            .try_code(predicate::eq(0))
    })
    .unwrap();

    assert.stderr(predicate::str::contains(
        "Warning: Command failed for target panicky",
    ));
}
