use cargo_test_fuzz::TestFuzz;
use internal::dirs::workspace;
use snapbox::{Redactions, assert_data_eq};
use std::{io::Write, process::Command};
use testing::CommandExt;

// smoelius: We need the `WS` redaction because we cannot assume the `test-fuzz` repository was
// cloned into a directory named `test-fuzz`.
const EXPECTED_SHOW_ENV: &str = "\
RUSTFLAGS='-C instrument-coverage --cfg=coverage --cfg=trybuild_no_target'
LLVM_PROFILE_FILE='[TARGET_DIR]/[WS]-%p-%16m.profraw'
CARGO_LLVM_COV=1
CARGO_LLVM_COV_SHOW_ENV=1
CARGO_LLVM_COV_TARGET_DIR=[TARGET_DIR]
CARGO_LLVM_COV_BUILD_DIR=[TARGET_DIR]
";

#[test]
fn llvm_cov_show_env() {
    let mut command = Command::new("which");
    command.arg("cargo-llvm-cov");
    let output = command.output().unwrap();
    if !output.status.success() {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "Skipping `llvm_cov_show_env` test as `cargo-llvm-cov` is unavailable"
        )
        .unwrap();
        return;
    }
    let stdout = String::from_utf8(output.stdout).unwrap();
    let assert = Command::new(stdout.trim_end())
        .args(["llvm-cov", "show-env"])
        .logged_assert();
    let actual_show_env = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let mut redactions = Redactions::new();
    redactions
        .insert("[TARGET_DIR]", TestFuzz::default().target_directory())
        .unwrap();
    redactions.insert("[WS]", workspace()).unwrap();
    assert_data_eq!(redactions.redact(actual_show_env), EXPECTED_SHOW_ENV);
}
