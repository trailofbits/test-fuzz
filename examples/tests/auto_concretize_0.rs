#![cfg(feature = "__auto_concretize")]

#[macro_export]
macro_rules! declarations {
    () => {
        use serde::{de::DeserializeOwned, Deserialize, Serialize};

        #[derive(Clone, Deserialize, Serialize)]
        struct Foo<T>(T);

        #[test_fuzz::test_fuzz_impl]
        impl<T: Clone + DeserializeOwned + Serialize> Foo<T> {
            #[test_fuzz::test_fuzz]
            fn target<U: Clone + DeserializeOwned + Serialize>(x: &U) {}
        }

        #[derive(Clone, Deserialize, Serialize)]
        pub(crate) struct Bar;

        #[derive(Clone, Deserialize, Serialize)]
        pub(crate) struct Baz;

        #[test]
        fn test() {
            Foo::<Bar>::target(&Baz)
        }
    };
}

declarations!();

mod auto_concretize_1;
