use testing::{examples, CommandExt};

#[test]
fn build_no_instrumentation() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--no-instrumentation"])
        .logged_assert()
        .success();
}

#[test]
fn build() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run"])
        .logged_assert()
        .success();
}

#[test]
fn build_pesistent() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--persistent"])
        .logged_assert()
        .success();
}
