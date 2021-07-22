use internal::{dirs::corpus_directory_from_target, examples};
use predicates::prelude::*;
use rlimit::{Resource, Rlim};
use std::fs::remove_dir_all;

// smoelius: MEMORY_LIMIT must be large enough for the build process to complete.
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

const TIMEOUT: &str = "120";

enum What {
    Crashes,
    Hangs,
}

#[test]
fn replay_crashes() {
    replay(
        "alloc",
        "alloc::target",
        &[
            "--run-until-crash",
            "--",
            "-m",
            &format!("{}", MEMORY_LIMIT / 1024),
        ],
        &What::Crashes,
        r"memory allocation of \d{10,} bytes failed\n",
    )
}

#[allow(clippy::trivial_regex)]
#[test]
fn replay_hangs() {
    replay(
        "parse_duration",
        "parse_duration::parse",
        &["--persistent", "--", "-V", TIMEOUT],
        &What::Hangs,
        r"Timeout\n",
    )
}

fn replay(name: &str, target: &str, fuzz_args: &[&str], what: &What, re: &str) {
    let corpus = corpus_directory_from_target(name, target);

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(name, &format!("{}::test", name))
        .unwrap()
        .assert()
        .success();

    examples::test_fuzz(target)
        .unwrap()
        .args(&["--reset"])
        .assert()
        .success();

    let mut args = vec!["--no-ui"];
    args.extend_from_slice(fuzz_args);

    examples::test_fuzz(target)
        .unwrap()
        .args(args)
        .assert()
        .success();

    // smoelius: The memory limit must be set to replay the crashes, but not the hangs.
    Resource::DATA
        .set(Rlim::from_raw(MEMORY_LIMIT), Rlim::from_raw(MEMORY_LIMIT))
        .unwrap();

    let mut command = examples::test_fuzz(target).unwrap();

    command
        .args(&[match what {
            What::Crashes => "--replay-crashes",
            What::Hangs => "--replay-hangs",
        }])
        .assert()
        .success()
        .stdout(predicate::str::is_match(re).unwrap());
}
