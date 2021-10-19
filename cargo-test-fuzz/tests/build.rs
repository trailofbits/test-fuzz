use testing::examples;

// smoelius: In each use of `examples::test_fuzz` below, the `target` argument is arbitrary. To
// have a version of `examples::test_fuzz` that takes no arguments for just these tests would be
// more trouble than it is worth.

#[test]
fn build_no_instrumentation() {
    examples::test_fuzz("assert", "target")
        .unwrap()
        .args(&["--no-run", "--no-instrumentation"])
        .assert()
        .success();
}

#[test]
fn build() {
    examples::test_fuzz("assert", "target")
        .unwrap()
        .args(&["--no-run"])
        .assert()
        .success();
}

#[test]
fn build_pesistent() {
    examples::test_fuzz("assert", "target")
        .unwrap()
        .args(&["--no-run", "--persistent"])
        .assert()
        .success();
}
