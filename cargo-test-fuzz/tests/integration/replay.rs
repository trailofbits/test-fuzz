// smoelius: `rlimit` does not work on macOS.
#![cfg(not(target_os = "macos"))]

use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use rlimit::Resource;
use std::fs::remove_dir_all;
use testing::{examples, retry, unique_id, CommandExt};

// smoelius: MEMORY_LIMIT must be large enough for the build process to complete.
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

const MAX_TOTAL_TIME: &str = "240";

#[derive(Clone, Copy)]
enum Object {
    Crashes,
    Hangs,
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn replay_crashes() {
    replay(
        "alloc",
        "target",
        &[
            "--run-until-crash",
            "--",
            "-m",
            &format!("{}", MEMORY_LIMIT / 1024),
        ],
        Object::Crashes,
        r"\bmemory allocation of \d{10,} bytes failed\b|\bcapacity overflow\b",
    );
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[allow(clippy::trivial_regex)]
#[test]
fn replay_hangs() {
    replay(
        "parse_duration",
        "parse",
        &["--persistent", "--max-total-time", MAX_TOTAL_TIME],
        Object::Hangs,
        r"(?m)\bTimeout$",
    );
}

fn replay(krate: &str, target: &str, fuzz_args: &[&str], object: Object, re: &str) {
    let id = unique_id();

    let corpus = corpus_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(corpus).unwrap_or_default();

    examples::test(krate, "test")
        .unwrap()
        .logged_assert()
        .success();

    examples::test_fuzz(krate, target)
        .unwrap()
        .args(["--reset", "--", "-M", &id])
        .logged_assert()
        .success();

    retry(3, || {
        let mut args = vec!["--no-ui", "--", "-M", &id];
        args.extend_from_slice(fuzz_args);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(args)
            .logged_assert()
            .success();

        // smoelius: The memory limit must be set to replay the crashes, but not the hangs.
        Resource::DATA.set(MEMORY_LIMIT, MEMORY_LIMIT).unwrap();

        let mut command = examples::test_fuzz(krate, target).unwrap();

        command
            .args([match object {
                Object::Crashes => "--replay=crashes",
                Object::Hangs => "--replay=hangs",
            }])
            .args(["--", "-M", &id])
            .logged_assert()
            .success()
            .try_stdout(predicate::str::is_match(re).unwrap())
    })
    .unwrap();
}
