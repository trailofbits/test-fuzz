use predicates::prelude::*;
use std::process::Command;
use testing::{examples::MANIFEST_PATH, CommandExt};

#[test]
fn conversion() {
    let mut command = test();

    command.logged_assert().success();

    command
        .args(["--features", "__inapplicable_conversion"])
        .logged_assert()
        .failure()
        .stderr(predicate::str::is_match(r#"(?m)\bConversion "Y" -> "Z" does not apply to the following candidates: \{\s*"X",\s*}$"#).unwrap());
}

fn test() -> Command {
    let mut command = Command::new("cargo");
    command.env("CARGO_TERM_COLOR", "never");
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
