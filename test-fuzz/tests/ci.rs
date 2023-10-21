use assert_cmd::assert::OutputAssertExt;
use regex::Regex;
use similar_asserts::SimpleDiff;
use std::{
    env::{set_current_dir, set_var, var},
    fs::{read_to_string, write},
    path::Path,
    process::Command,
    str::FromStr,
};
use testing::CommandExt;

#[ctor::ctor]
fn initialize() {
    set_current_dir("..").unwrap();
    set_var("CARGO_TERM_COLOR", "never");
}

#[test]
fn readme_contains_usage() {
    let readme = read_to_string("README.md").unwrap();

    let assert = assert_cmd::Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .args(["test-fuzz", "--help"])
        .logged_assert();
    let stdout = &assert.get_output().stdout;

    let usage = std::str::from_utf8(stdout)
        .unwrap()
        .split_inclusive('\n')
        .skip(2)
        .collect::<String>();

    assert!(
        readme.contains(&usage),
        "{}",
        SimpleDiff::from_str(&readme, &usage, "left", "right")
    );
}

#[test]
fn readme_does_not_use_inline_links() {
    let readme = read_to_string("README.md").unwrap();
    assert!(
        !Regex::new(r"\[[^\]]*\]\(").unwrap().is_match(&readme),
        "readme uses inline links",
    );
}

#[test]
fn readme_reference_links_are_sorted() {
    let re = Regex::new(r"^\[[^\]]*\]:").unwrap();
    let readme = read_to_string("README.md").unwrap();
    let links = readme
        .lines()
        .filter(|line| re.is_match(line))
        .collect::<Vec<_>>();
    let mut links_sorted = links.clone();
    links_sorted.sort_unstable();
    assert_eq!(links_sorted, links);
}

// smoelius: No other test uses supply_chain.json.
#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
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
