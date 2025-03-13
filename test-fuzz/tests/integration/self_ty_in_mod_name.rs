use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use testing::examples::MANIFEST_PATH;

#[test]
fn success() {
    let mut command = test();

    command.assert().success();
}

#[test]
fn self_ty_conflict() {
    let mut command = test();

    command
        .args(["--features=__self_ty_conflict"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the name `struct_target_fuzz__` is defined multiple times",
        ));
}

fn test() -> Command {
    let mut command = Command::new("cargo");
    command.args([
        "test",
        "--manifest-path",
        &MANIFEST_PATH,
        "--no-run",
        "--features",
        &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
    ]);
    command
}
