include!(concat!(env!("OUT_DIR"), "/profile.rs"));

#[test_fuzz::test_fuzz]
fn target(x: bool) {
    assert!(!x || PROFILE != "release");
}

#[test]
fn test() {
    target(false);
}
