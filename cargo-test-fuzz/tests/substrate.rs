use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs::read_to_string, path::Path};
use tempfile::tempdir_in;

const SUBSTRATE_NODE_TEMPLATE_URL: &str =
    "https://github.com/substrate-developer-hub/substrate-node-template";

const PACKAGE: &str = "pallet-template";

#[test]
fn do_something() {
    // smoelius: `substrate_node_template.patch` expects test-fuzz to be an ancestor of the
    // directory in which the patch is applied.
    let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["clone", SUBSTRATE_NODE_TEMPLATE_URL, "."])
        .assert()
        .success();

    let patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("substrate_node_template.patch")
        .canonicalize()
        .unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["apply", &patch.to_string_lossy().to_string()])
        .assert()
        .success();

    Command::new("cargo")
        .current_dir(tempdir.path())
        .args(&["test", "--package", PACKAGE, "--", "--nocapture"])
        .assert()
        .success();

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(tempdir.path())
        .args(&["test-fuzz", "--package", PACKAGE, "--display-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"^[[:xdigit:]]{40}:"#).unwrap());

    Command::cargo_bin("cargo-test-fuzz")
        .unwrap()
        .current_dir(tempdir.path())
        .args(&["test-fuzz", "--package", PACKAGE, "--replay-corpus"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"^[[:xdigit:]]{40}: Ret\(Ok\(\(\)\)\)\n"#).unwrap());
}

#[test]
fn patch_is_current() {
    let tempdir = tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["clone", SUBSTRATE_NODE_TEMPLATE_URL, "."])
        .assert()
        .success();

    let patch_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("substrate_node_template.patch");
    let patch = read_to_string(patch_path).unwrap();

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["apply"])
        .write_stdin(patch.as_bytes())
        .assert()
        .success();

    let assert = Command::new("git")
        .current_dir(tempdir.path())
        .args(&["diff"])
        .assert()
        .success();

    let diff = String::from_utf8_lossy(&assert.get_output().stdout);

    assert_eq!(patch, diff);
}
