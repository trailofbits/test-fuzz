mod assert {
    #[test_fuzz::test_fuzz]
    pub fn target(x: bool) {
        assert!(!x);
    }

    #[test]
    fn test() {
        target(false);
    }
}
