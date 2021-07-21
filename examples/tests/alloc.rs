mod alloc {
    #[test_fuzz::test_fuzz(no_auto)]
    fn target(n: usize) {
        let _vec = Vec::<u8>::with_capacity(n);
    }

    #[test]
    fn test() {
        target(0);
    }
}
