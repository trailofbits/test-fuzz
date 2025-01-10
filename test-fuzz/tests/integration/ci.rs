use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use regex::Regex;
use similar_asserts::SimpleDiff;
use std::{
    env::var,
    fs::{read_to_string, write},
    path::Path,
    process::Command,
    str::FromStr,
};
use testing::CommandExt;

#[test]
fn clippy() {
    Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "--deny=warnings"])
        .assert()
        .success();
}

#[test]
fn license() {
    let re = Regex::new(r"^[^:]*\b(Apache-2.0|BSD-2-Clause|BSD-3-Clause|ISC|MIT|N/A)\b").unwrap();

    for line in std::str::from_utf8(
        &Command::new("cargo")
            .arg("license")
            .current_dir("..")
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .unwrap()
    .lines()
    {
        if line
            == "AGPL-3.0 WITH mif-exception (5): cargo-test-fuzz, test-fuzz, test-fuzz-internal, \
                test-fuzz-macro, test-fuzz-runtime"
        {
            continue;
        }
        assert!(re.is_match(line), "{line:?} does not match");
    }
}

#[test]
fn readme_contains_usage() {
    let readme = read_to_string("../README.md").unwrap();

    let assert = Command::cargo_bin("cargo-test-fuzz")
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
    let readme = read_to_string("../README.md").unwrap();
    assert!(
        !Regex::new(r"\[[^\]]*\]\(").unwrap().is_match(&readme),
        "readme uses inline links",
    );
}

#[test]
fn readme_reference_links_are_sorted() {
    let re = Regex::new(r"^\[[^\]]*\]:").unwrap();
    let readme = read_to_string("../README.md").unwrap();
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

    let path = Path::new("tests/supply_chain.json");

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

#[test]
fn udeps() {
    Command::new("cargo")
        .args([
            "+nightly",
            "udeps",
            "--features=test-install",
            "--all-targets",
        ])
        .current_dir("..")
        .assert()
        .success();
}

#[test]
fn unmaintained() {
    Command::new("cargo")
        .args(["unmaintained", "--color=never", "--fail-fast"])
        .assert()
        .success();
}

#[must_use]
pub fn enabled(key: &str) -> bool {
    var(key).is_ok_and(|value| value != "0")
}
