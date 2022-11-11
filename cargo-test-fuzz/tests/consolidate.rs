use anyhow::ensure;
use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};
use test_log::test;
use testing::{examples, retry};

const CRASH_TIMEOUT: &str = "60";

const HANG_TIMEOUT: &str = "120";

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn consolidate_crashes() {
    consolidate(
        "assert",
        "target",
        &["--run-until-crash", "--", "-V", CRASH_TIMEOUT],
        "Args { x: true }",
    );
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn consolidate_hangs() {
    consolidate(
        "parse_duration",
        "parse",
        &["--persistent", "--", "-V", HANG_TIMEOUT],
        "",
    );
}

fn consolidate(krate: &str, target: &str, fuzz_args: &[&str], pattern: &str) {
    let corpus = corpus_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(krate, "test").unwrap().assert().success();

    assert_eq!(read_dir(&corpus).unwrap().count(), 1);

    retry(3, || {
        let mut args = vec!["--no-ui"];
        args.extend_from_slice(fuzz_args);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(args)
            .assert()
            .success();

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(["--consolidate"])
            .assert()
            .success();

        ensure!(read_dir(&corpus).unwrap().count() > 1);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(["--display=corpus"])
            .assert()
            .success()
            .try_stdout(predicate::str::contains(pattern))
            .map_err(Into::into)
    })
    .unwrap();
}
