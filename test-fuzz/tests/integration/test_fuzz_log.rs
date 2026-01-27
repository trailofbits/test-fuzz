use predicates::prelude::*;
use std::process::Command;
use testing::{CommandExt, fuzzable::MANIFEST_PATH};

// smoelius: This test will fail if run twice because the target will have already been built.
#[test]
fn test_fuzz_log() {
    Command::new("cargo")
        .env("TEST_FUZZ_LOG", "1")
        .args([
            "test",
            "--manifest-path",
            MANIFEST_PATH,
            "--no-run",
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
            "--test=parse_duration",
        ])
        .logged_assert()
        .success()
        .stdout(predicate::str::is_match(r"(?m)^#\[cfg\(test\)\]\nmod parse_fuzz__ \{$").unwrap());
}
