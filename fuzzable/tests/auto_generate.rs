mod signed {
    #[test_fuzz::test_fuzz]
    fn target(x: i64) {}

    #[test]
    fn test() {
        target(i64::default());
    }
}

mod unsigned {
    #[test_fuzz::test_fuzz]
    fn target(x: u64) {}

    #[test]
    fn test() {
        target(u64::default());
    }
}
