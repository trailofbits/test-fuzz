mod alloc {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    pub fn target(n: usize) {
        let _ = Vec::<u8>::with_capacity(n);
    }

    #[test]
    fn test() {
        target(0);
    }
}
