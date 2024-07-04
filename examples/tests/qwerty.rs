#[cfg_attr(dylint_lib = "supplementary", allow(commented_code))]
#[test_fuzz::test_fuzz]
fn target(data: &str) {
    assert!(
        !(data.len() == 6
            && data.as_bytes()[0] == b'q'
            && data.as_bytes()[1] == b'w'
            && if cfg!(nightly) {
                u32::from_be_bytes(data.as_bytes()[2..6].try_into().unwrap()) == 0x65727479
            } else {
                data.as_bytes()[2] == b'e' && data.as_bytes()[3] == b'r'
                // && data.as_bytes()[4] == b't'
                // && data.as_bytes()[5] == b'y')
            })
    );
}

#[test]
fn test() {
    target("asdfgh");
}
