use assert_cmd::assert::OutputAssertExt;
use similar_asserts::SimpleDiff;
use std::{
    env::{set_current_dir, set_var, var},
    fs::{read_to_string, write},
    path::Path,
    process::Command,
    str::FromStr,
};

#[ctor::ctor]
fn initialize() {
    set_current_dir("..").unwrap();
    set_var("CARGO_TERM_COLOR", "never");
}

#[test]
fn supply_chain() {
    Command::new("cargo")
        .args(["supply-chain", "update"])
        .assert()
        .success();

    let assert = Command::new("cargo")
        .args(["supply-chain", "json", "--no-dev"])
        .assert()
        .success();

    let stdout_actual = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let value = serde_json::Value::from_str(stdout_actual).unwrap();
    let stdout_normalized = serde_json::to_string_pretty(&value).unwrap() + "\n";

    let path = Path::new("test-fuzz/tests/supply_chain.json");

    let stdout_expected = read_to_string(path).unwrap();

    if enabled("BLESS") {
        write(path, stdout_normalized).unwrap();
    } else {
        assert!(
            stdout_expected == stdout_normalized,
            "{}",
            SimpleDiff::from_str(&stdout_expected, &stdout_normalized, "left", "right")
        );
    }
}

#[must_use]
pub fn enabled(key: &str) -> bool {
    var(key).map_or(false, |value| value != "0")
}
