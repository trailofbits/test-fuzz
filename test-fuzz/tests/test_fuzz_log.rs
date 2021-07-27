use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::path::Path;

lazy_static! {
    static ref MANIFEST_PATH: String = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("Cargo.toml")
        .to_string_lossy()
        .to_string();
}

// smoelius: This test will fail if run twice because the target will have already been built.
#[test]
fn test_fuzz_log() {
    Command::new("cargo")
        .envs(vec![("TEST_FUZZ_LOG", "1")])
        .args(&[
            "test",
            "--manifest-path",
            &MANIFEST_PATH,
            "--no-run",
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format().as_feature()),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\n#\[cfg\(test\)\]\nmod parse_fuzz \{\n").unwrap());
}
