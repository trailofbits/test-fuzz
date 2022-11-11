use test_log::test;
use testing::examples;

#[test]
fn build_no_instrumentation() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--no-instrumentation"])
        .assert()
        .success();
}

#[test]
fn build() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run"])
        .assert()
        .success();
}

#[test]
fn build_pesistent() {
    examples::test_fuzz_all()
        .unwrap()
        .args(["--no-run", "--persistent"])
        .assert()
        .success();
}
