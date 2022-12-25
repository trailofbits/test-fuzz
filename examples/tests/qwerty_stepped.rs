#[test_fuzz::test_fuzz]
fn target(data: &str) {
    assert!(
        !(data.len() == 6
            && (cfg!(feature = "__qwerty_q") || data.as_bytes()[0] == b'q')
            && (cfg!(feature = "__qwerty_w") || data.as_bytes()[1] == b'w')
            && (cfg!(feature = "__qwerty_e") || data.as_bytes()[2] == b'e')
            && (cfg!(feature = "__qwerty_r") || data.as_bytes()[3] == b'r')
            && (cfg!(feature = "__qwerty_t") || data.as_bytes()[4] == b't')
            && (cfg!(feature = "__qwerty_y") || data.as_bytes()[5] == b'y'))
    );
}

#[test]
fn test() {
    target("asdfgh");
}
