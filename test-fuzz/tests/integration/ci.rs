use assert_cmd::cargo::CommandCargoExt;
use regex::Regex;
use similar_asserts::SimpleDiff;
use std::{
    collections::HashSet,
    env::var,
    ffi::OsStr,
    fs::{read_dir, read_to_string, write},
    path::Path,
    process::Command,
    str::FromStr,
};
use tempfile::tempdir;
use testing::CommandExt;
use walkdir::WalkDir;

#[test]
fn clippy() {
    Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "--deny=warnings"])
        .logged_assert()
        .success();
}

#[test]
fn format() {
    Command::new("cargo")
        .args(["+nightly", "fmt", "--check"])
        .current_dir("..")
        .logged_assert()
        .success();
}

#[test]
fn shellcheck() {
    for entry in read_dir("../scripts").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            Command::new("shellcheck")
                .args(["--exclude=SC2002", &path.to_string_lossy()])
                .logged_assert()
                .success();
        }
    }
}


#[test]
fn dylint() {
    Command::new("cargo")
        .args(["dylint", "--all", "--", "--all-targets"])
        .env("DYLINT_RUSTFLAGS", "--deny warnings")
        .logged_assert()
        .success();
}

#[test]
fn cargo_sort() {
    for entry in WalkDir::new("..")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == OsStr::new("Cargo.toml"))
    {
        let dir = entry.path().parent().unwrap();
        Command::new("cargo")
            .args(["sort", "--check", "--grouped", "--no-format"])
            .current_dir(dir)
            .logged_assert()
            .success();
    }
}

#[test]
fn license() {
    let re = Regex::new(r"^[^:]*\b(Apache-2.0|BSD-2-Clause|BSD-3-Clause|ISC|MIT|N/A)\b").unwrap();

    for line in std::str::from_utf8(
        &Command::new("cargo")
            .arg("license")
            .current_dir("..")
            .logged_assert()
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
        .logged_assert()
        .success();

    let assert = Command::new("cargo")
        .args(["supply-chain", "json", "--no-dev"])
        .logged_assert()
        .success();

    let stdout_actual = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let mut value_actual = serde_json::Value::from_str(stdout_actual).unwrap();
    remove_avatars(&mut value_actual);
    let stdout_normalized = serde_json::to_string_pretty(&value_actual).unwrap() + "\n";

    let path = Path::new("tests/supply_chain.json");

    if enabled("BLESS") {
        write(path, stdout_normalized).unwrap();
    } else {
        let object_actual = value_actual.as_object().unwrap();
        let set_actual = object_actual.into_iter().collect::<HashSet<_>>();

        let stdout_expected = read_to_string(path).unwrap();
        let value_expected = serde_json::Value::from_str(&stdout_expected).unwrap();
        let object_expected = value_expected.as_object().unwrap();
        let set_expected = object_expected.into_iter().collect::<HashSet<_>>();

        // smoelius: Fail only if actual is not a subset of expected.
        assert!(
            set_actual.is_subset(&set_expected),
            "{}",
            SimpleDiff::from_str(&stdout_expected, &stdout_normalized, "left", "right")
        );
    }
}

fn remove_avatars(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
        serde_json::Value::Array(array) => {
            for value in array {
                remove_avatars(value);
            }
        }
        serde_json::Value::Object(object) => {
            object.retain(|key, value| {
                if key == "avatar" {
                    return false;
                }
                remove_avatars(value);
                true
            });
        }
    }
}

#[test]
fn markdown_link_check() {
    let tempdir = tempdir().unwrap();

    Command::new("npm")
        .args(["install", "markdown-link-check"])
        .current_dir(&tempdir)
        .logged_assert()
        .success();

    let readme_md = Path::new(env!("CARGO_MANIFEST_DIR")).join("../README.md");

    Command::new("npx")
        .args(["markdown-link-check", readme_md.to_str().unwrap()])
        .current_dir(&tempdir)
        .logged_assert()
        .success();
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
        .logged_assert()
        .success();
}

#[test]
fn unmaintained() {
    Command::new("cargo")
        .args(["unmaintained", "--color=never", "--fail-fast"])
        .logged_assert()
        .success();
}

#[must_use]
pub fn enabled(key: &str) -> bool {
    var(key).is_ok_and(|value| value != "0")
}
