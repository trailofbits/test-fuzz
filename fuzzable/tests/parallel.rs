#[test_fuzz::test_fuzz]
fn target_0(x: bool) {}

#[test_fuzz::test_fuzz]
fn target_1(x: bool) {}

#[test_fuzz::test_fuzz]
fn target_2(x: bool) {}

#[test_fuzz::test_fuzz]
fn target_3(x: bool) {
    assert!(!x);
}

#[test_fuzz::test_fuzz]
fn target_4(x: bool) {}

#[test_fuzz::test_fuzz]
fn target_5(x: bool) {}

#[test]
fn test() {
    target_0(false);
    target_1(false);
    target_2(false);
    target_3(false);
    target_4(false);
    target_5(false);
}
