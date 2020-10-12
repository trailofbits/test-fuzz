mod assert {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    pub fn target(x: bool) {
        assert!(x);
    }

    #[test]
    fn test() {
        target(true);
    }
}
