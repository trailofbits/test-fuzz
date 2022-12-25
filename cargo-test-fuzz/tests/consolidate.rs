use anyhow::ensure;
use internal::{dirs::corpus_directory_from_target, fuzzer, Fuzzer};
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};
use testing::{examples, retry, CommandExt};

const CRASH_MAX_TOTAL_TIME: &str = "60";

const HANG_MAX_TOTAL_TIME: &str = "120";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn consolidate_crashes() {
    consolidate(
        "assert",
        "target",
        &[
            "--run-until-crash",
            "--max-total-time",
            CRASH_MAX_TOTAL_TIME,
        ],
        1,
        "Args { x: true }",
    );
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn consolidate_hangs() {
    let fuzzer = fuzzer().unwrap();
    consolidate(
        "parse_duration",
        "parse",
        &["--max-total-time", HANG_MAX_TOTAL_TIME],
        match fuzzer {
            Fuzzer::Aflplusplus | Fuzzer::AflplusplusPersistent => 0,
            Fuzzer::Libfuzzer => 1,
        },
        "",
    );
}

fn consolidate(krate: &str, target: &str, fuzz_args: &[&str], code: i32, pattern: &str) {
    let corpus = corpus_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(krate, "test")
        .unwrap()
        .logged_assert()
        .success();

    assert_eq!(read_dir(&corpus).unwrap().count(), 1);

    retry(3, || {
        let mut args = vec!["--exit-code", "--no-ui"];
        args.extend_from_slice(fuzz_args);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(args)
            .logged_assert()
            .try_code(code)?;

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(["--consolidate"])
            .logged_assert()
            .success();

        ensure!(read_dir(&corpus).unwrap().count() > 1);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(["--display=corpus"])
            .logged_assert()
            .success()
            .try_stdout(predicate::str::contains(pattern))
            .map_err(Into::into)
    })
    .unwrap();
}
