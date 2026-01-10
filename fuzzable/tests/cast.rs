#[allow(clippy::cast_possible_truncation)]
#[test_fuzz::test_fuzz]
fn target(x: u64) {
    let _ = x as u32;
}

#[test]
fn test() {
    target(0);
}
