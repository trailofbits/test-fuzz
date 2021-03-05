mod alloc {
    #[test_fuzz::test_fuzz]
    fn target(n: usize) {
        let _ = Vec::<u8>::with_capacity(n);
    }

    #[test]
    fn test() {
        target(0);
    }
}
