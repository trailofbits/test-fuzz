use assert_cmd::prelude::*;
use dirs::corpus_directory_from_target;
use std::{fs::read_dir, process::Command};

const TEST_DIR: &str = "../examples";

#[test]
fn test_no_default() {
    test("no_default", 1)
}

#[test]
fn test_default() {
    test("default", 2)
}

fn test(target: &str, n: usize) {
    Command::new("cargo")
        .current_dir(TEST_DIR)
        .args(&["test", "--", "--test", target])
        .assert()
        .success();

    let dir = corpus_directory_from_target("default", &(target.to_owned() + "::target"));

    assert_eq!(read_dir(dir).unwrap().count(), n);
}
