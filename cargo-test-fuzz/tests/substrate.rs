use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs::File, io::Read, path::Path};
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

    // smoelius: I was getting sporadic `unrecognized input` errors from `git apply` in CI. I don't
    // know why. Writing the patch directly to `git apply`'s standard input seems to make the
    // problem go away.
    // smoelius: The problem persists. For some reason, the patch appears empty, even though
    // `read_to_string` succeeds. I am going to see if using `File::read_to_end` makes a difference.
    let patch_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("substrate_node_template.patch");
    let mut file = File::open(patch_path).unwrap();
    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf).unwrap();
    let patch = String::from_utf8(buf).unwrap();
    println!("{}", patch);

    Command::new("git")
        .current_dir(tempdir.path())
        .args(&["apply"])
        .write_stdin(patch)
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
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    Command::new(
        parent
            .join("scripts")
            .join("update_substrate_node_template_patch.sh"),
    )
    .current_dir(parent)
    .assert()
    .success();

    Command::new("git")
        .current_dir(parent)
        .args(&[
            "diff",
            "--exit-code",
            "--",
            "update_substrate_node_template.patch",
        ])
        .assert()
        .success();
}
