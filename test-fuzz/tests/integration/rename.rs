use predicates::prelude::*;
use std::process::Command;
use testing::{examples::MANIFEST_PATH, CommandExt};

#[test]
fn rename() {
    let mut command = test();

    command.logged_assert().success();

    command
        .args(["--features", "__bar_fuzz"])
        .logged_assert()
        .failure()
        .stderr(predicate::str::contains(
            "the name `bar_fuzz__` is defined multiple times",
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
