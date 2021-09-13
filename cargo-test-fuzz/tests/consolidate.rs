use anyhow::ensure;
use internal::{dirs::corpus_directory_from_target, examples, testing::retry};
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};
use test_env_log::test;

const CRASH_TIMEOUT: &str = "60";

const HANG_TIMEOUT: &str = "120";

#[test]
fn consolidate_crashes() {
    consolidate(
        "assert",
        "assert::target",
        &["--run-until-crash", "--", "-V", CRASH_TIMEOUT],
        "Args { x: true }",
    );
}

#[test]
fn consolidate_hangs() {
    consolidate(
        "parse_duration",
        "parse_duration::parse",
        &["--persistent", "--", "-V", HANG_TIMEOUT],
        "",
    );
}

fn consolidate(name: &str, target: &str, fuzz_args: &[&str], pattern: &str) {
    let corpus = corpus_directory_from_target(name, target);

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(name, &format!("{}::test", name))
        .unwrap()
        .assert()
        .success();

    assert_eq!(read_dir(&corpus).unwrap().count(), 1);

    retry(3, || {
        let mut args = vec!["--no-ui"];
        args.extend_from_slice(fuzz_args);

        examples::test_fuzz(name, target)
            .unwrap()
            .args(args)
            .assert()
            .success();

        examples::test_fuzz(name, target)
            .unwrap()
            .args(&["--consolidate"])
            .assert()
            .success();

        ensure!(read_dir(&corpus).unwrap().count() > 1);

        examples::test_fuzz(name, target)
            .unwrap()
            .args(&["--display-corpus"])
            .assert()
            .success()
            .try_stdout(predicate::str::contains(pattern))
            .map_err(Into::into)
    })
    .unwrap();
}
