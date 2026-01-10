#[test_fuzz::test_fuzz]
fn target(x: bool) {
    assert!(!x);
}

#[test]
fn test() {
    target(false);
}
