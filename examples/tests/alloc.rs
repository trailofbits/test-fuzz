mod alloc {
    #[test_fuzz::test_fuzz(no_auto_generate)]
    fn target(n: usize) {
        let vec = Vec::<u8>::with_capacity(n);
        // smoelius: Prevent `vec` from being optimized away.
        println!("{:p}", &vec);
    }

    #[test]
    fn test() {
        target(0);
    }
}
