use internal::dirs::output_directory_from_target;
use predicates::prelude::*;
use std::{ffi::OsStr, fs::remove_dir_all};
use testing::{examples, retry, CommandExt};

const CPUS: &str = "2";
const TIME_SLICE: &str = "30";

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
