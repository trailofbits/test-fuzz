use internal::dirs::target_directory;
use predicates::prelude::*;
use std::process::Command;
use testing::{LoggedAssert, fuzzable::MANIFEST_PATH};

const SERDE_DEFAULT: &str = "bincode";

#[test]
fn link() {
    Command::new("cargo")
        .args([
            "build",
            "--manifest-path",
            MANIFEST_PATH,
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
        ])
        .logged_assert()
        .success();

    let pred = predicate::str::contains(SERDE_DEFAULT);

    #[cfg(not(any(serde_default, feature = "serde_bincode")))]
    let pred = pred.not();

    // smoelius: https://stackoverflow.com/questions/7219845/difference-between-nm-and-objdump
    #[allow(clippy::disallowed_methods)]
    Command::new("nm")
        .args([target_directory(false, false)
            .join("debug/hello-world")
            .to_string_lossy()
            .to_string()])
        .logged_assert()
        .success()
        .stdout(pred);
}
