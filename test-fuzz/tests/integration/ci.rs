use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use regex::Regex;
use similar_asserts::SimpleDiff;
use std::{
    env,
    ffi::OsStr,
    fs::{read_dir, read_to_string, write},
    io::Write,
    path::Path,
    process::{Command, ExitStatus},
    str::FromStr,
};
use tempfile::tempdir;
use testing::CommandExt;
use walkdir::WalkDir;

#[test]
fn actionlint() {
    let mut command = Command::new("which");
    command.arg("actionlint");
    let output = command.output().unwrap();
    if !output.status.success() {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "Skipping `actionlint` test as `actionlint` is unavailable"
        )
        .unwrap();
        return;
    }
    let stdout = String::from_utf8(output.stdout).unwrap();
    Command::new(stdout.trim_end())
        .env("SHELLCHECK_OPTS", "--exclude=SC2002")
        .logged_assert()
        .success();
}

#[test]
fn clippy() {
    Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "--deny=warnings"])
        .logged_assert()
        .success();
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
fn fmt() {
    Command::new("cargo")
        .args(["+nightly", "fmt", "--check"])
        .current_dir("..")
        .logged_assert()
        .success();
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
fn prettier() {
    Command::new("prettier")
        .args([
            "--check",
            "**/*.json",
            "**/*.md",
            "**/*.yml",
            "!test-fuzz/tests/supply_chain.json",
        ])
        .current_dir("..")
        .logged_assert()
        .success();
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
fn sort() {
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

// smoelius: No other test uses supply_chain.json.
#[allow(clippy::disallowed_methods)]
#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn supply_chain() {
    let mut command = Command::new("cargo");
    command.args(["supply-chain", "update", "--cache-max-age=0s"]);
    let _: ExitStatus = command.status().unwrap();

    let mut command = Command::new("cargo");
    command.args(["supply-chain", "json", "--no-dev"]);
    let assert = command.assert().success();

    let stdout_actual = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let mut value = serde_json::Value::from_str(stdout_actual).unwrap();
    remove_avatars(&mut value);
    let stdout_normalized = serde_json::to_string_pretty(&value).unwrap();

    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/supply_chain.json");

    if enabled("BLESS") {
        write(path_buf, stdout_normalized).unwrap();
    } else {
        let stdout_expected = read_to_string(&path_buf).unwrap();

        assert!(
            stdout_expected == stdout_normalized,
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
    env::var(key).is_ok_and(|value| value != "0")
}
