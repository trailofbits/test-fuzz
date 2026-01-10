use testing::{CommandExt, fuzzable};

#[test]
fn build() {
    fuzzable::test_fuzz_all()
        .unwrap()
        .args(["--no-run"])
        .logged_assert()
        .success();
}

#[test]
fn build_persistent() {
    fuzzable::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--persistent"])
        .logged_assert()
        .success();
}
