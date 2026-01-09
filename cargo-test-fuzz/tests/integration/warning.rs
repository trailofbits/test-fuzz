use predicates::prelude::*;
use testing::{LoggedAssert, fuzzable::MANIFEST_PATH};

#[test]
fn warning() {
    #[allow(clippy::disallowed_methods)]
    std::process::Command::new("cargo")
        .args([
            "test",
            "--manifest-path",
            MANIFEST_PATH,
            "--no-run",
            "--features",
            &("test-fuzz/".to_owned() + internal::serde_format::as_feature()),
            "--features=__no_test_fuzz",
            "--test",
            "no_test_fuzz",
        ])
        .logged_assert()
        .success()
        .stderr(predicate::str::contains(
            "Warning: No `test_fuzz` attributes found in `impl` block",
        ));
}
