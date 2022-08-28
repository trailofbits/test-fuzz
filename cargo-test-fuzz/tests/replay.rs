use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use rlimit::Resource;
use std::fs::remove_dir_all;
use test_log::test;
use testing::{examples, retry};

// smoelius: MEMORY_LIMIT must be large enough for the build process to complete.
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

const TIMEOUT: &str = "120";

#[derive(Clone, Copy)]
enum Object {
    Crashes,
    Hangs,
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
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
        r"(?m)\bmemory allocation of \d{10,} bytes failed$|\bcapacity overflow\b",
    );
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[allow(clippy::trivial_regex)]
#[test]
fn replay_hangs() {
    replay(
        "parse_duration",
        "parse",
        &["--persistent", "--", "-V", TIMEOUT],
        Object::Hangs,
        r"(?m)\bTimeout$",
    );
}

fn replay(krate: &str, target: &str, fuzz_args: &[&str], object: Object, re: &str) {
    let corpus = corpus_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(krate, "test").unwrap().assert().success();

    examples::test_fuzz(krate, target)
        .unwrap()
        .args(&["--reset"])
        .assert()
        .success();

    retry(3, || {
        let mut args = vec!["--no-ui"];
        args.extend_from_slice(fuzz_args);

        examples::test_fuzz(krate, target)
            .unwrap()
            .args(args)
            .assert()
            .success();

        // smoelius: The memory limit must be set to replay the crashes, but not the hangs.
        Resource::DATA.set(MEMORY_LIMIT, MEMORY_LIMIT).unwrap();

        let mut command = examples::test_fuzz(krate, target).unwrap();

        command
            .args(&[match object {
                Object::Crashes => "--replay=crashes",
                Object::Hangs => "--replay=hangs",
            }])
            .assert()
            .success()
            .try_stdout(predicate::str::is_match(re).unwrap())
    })
    .unwrap();
}
