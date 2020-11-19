mod qwerty {
    #[test_fuzz::test_fuzz]
    pub fn target(data: &str) {
        assert!(
            !(data.len() == 6
                && data.as_bytes()[0] == b'q'
                && data.as_bytes()[1] == b'w'
                && data.as_bytes()[2] == b'e'
                && data.as_bytes()[3] == b'r'
                && data.as_bytes()[4] == b't'
                && data.as_bytes()[5] == b'y')
        );
    }

    #[test]
    fn test() {
        target("asdfgh");
    }
}
