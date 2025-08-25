use testing::{CommandExt, examples};

#[test]
fn build() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run"])
        .logged_assert()
        .success();
}

#[test]
fn build_persistent() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--persistent"])
        .logged_assert()
        .success();
}
