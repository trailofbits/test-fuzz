mod unserde {
    // smoelius: `Struct` is *not* serializable/deserializable.
    struct Struct;

    #[test_fuzz::test_fuzz(only_concretizations)]
    fn target<T>(_: &T) {}

    #[test_fuzz::test_fuzz(enable_in_production, only_concretizations)]
    fn target_in_production<T>(_: &T) {}

    #[test]
    fn test() {
        target(&Struct);
    }

    #[test]
    fn test_in_production() {
        target_in_production(&Struct);
    }
}
