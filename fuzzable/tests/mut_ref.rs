#[test_fuzz::test_fuzz]
fn target_slice(xs: &mut [u8]) {
    if !xs.is_empty() {
        xs[0] = 1;
    }
}

#[test_fuzz::test_fuzz]
fn target_str(s: &mut str) {
    s.make_ascii_uppercase();
}

#[test]
fn test() {
    target_slice(&mut [0]);
    target_str(String::from("x").as_mut_str());
}
