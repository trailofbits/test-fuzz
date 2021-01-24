mod no_default {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Deserialize, Serialize)]
    struct Struct(bool);

    #[test_fuzz::test_fuzz]
    fn target(s: &Struct) {}

    #[test]
    fn test() {
        target(&Struct(true));
    }
}

mod default {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct Struct(bool);

    #[test_fuzz::test_fuzz]
    fn target(s: &Struct) {}

    #[test]
    fn test() {
        target(&Struct(true));
    }
}
