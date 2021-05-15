#![allow(clippy::blacklisted_name)]

mod generic {
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    trait FooBarTrait {}

    // smoelius: `Foo` is serializable, but not deserializable.
    #[derive(Clone, Debug, Serialize)]
    enum Foo {
        A(String),
    }

    impl FooBarTrait for Foo {}

    #[derive(Clone, Debug, Deserialize, Serialize)]
    enum Bar {
        B(String),
    }

    impl FooBarTrait for Bar {}

    trait BazTrait {}

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct Baz<T>(T);

    impl<T> BazTrait for Baz<T> {}

    trait Trait<T: FooBarTrait> {
        fn target(&self, x: &T);
        fn target_bound<U: BazTrait + Clone + Debug + Serialize>(&self, x: &T, y: &U);
        fn target_where_clause<U>(&self, x: &T, y: &U)
        where
            U: BazTrait + Clone + Debug + Serialize;
        fn target_only_concretizations<U>(&self, x: &T, y: &U);
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct Struct;

    #[test_fuzz::test_fuzz_impl]
    impl<T: FooBarTrait + Clone + Debug + Serialize> Trait<T> for Struct {
        // smoelius: The Rust docs (https://doc.rust-lang.org/std/fmt/trait.Debug.html#stability)
        // state:
        // Derived `Debug` formats are not stable, and so may change with future Rust versions.
        // So `x` should not be compared to a string constant.
        #[test_fuzz::test_fuzz(concretize_impl = "Bar")]
        fn target(&self, x: &T) {
            assert_ne!(
                format!("{:?}", x),
                format!("{:?}", Bar::B("qwerty".to_owned()))
            );
        }

        #[test_fuzz::test_fuzz(concretize_impl = "Bar", concretize = "Baz<Bar>")]
        fn target_bound<U: BazTrait + Clone + Debug + Serialize>(&self, x: &T, y: &U) {}

        #[test_fuzz::test_fuzz(concretize_impl = "Bar", concretize = "Baz<Bar>")]
        fn target_where_clause<U>(&self, x: &T, y: &U)
        where
            U: BazTrait + Clone + Debug + Serialize,
        {
        }

        #[test_fuzz::test_fuzz(only_concretizations)]
        fn target_only_concretizations<U>(&self, _: &T, _: &U) {}
    }

    #[cfg(test)]
    lazy_static! {
        static ref FOO: Foo = Foo::A("qwerty".to_owned());
        static ref BAR: Bar = Bar::B("asdfgh".to_owned());
    }

    #[test]
    fn test_foo_qwerty() {
        Struct.target(&*FOO);
    }

    #[test]
    fn test_bar_asdfgh() {
        Struct.target(&*BAR);
    }

    #[test]
    fn test_bound() {
        Struct.target_bound(&*FOO, &Baz(FOO.clone()));
        Struct.target_bound(&*BAR, &Baz(BAR.clone()));
    }

    #[test]
    fn test_where_clause() {
        Struct.target_where_clause(&*FOO, &Baz(FOO.clone()));
        Struct.target_where_clause(&*BAR, &Baz(BAR.clone()));
    }

    #[test]
    fn test_only_concretizations() {
        Struct.target_only_concretizations(&*FOO, &Baz(FOO.clone()));
        Struct.target_only_concretizations(&*BAR, &Baz(BAR.clone()));
    }
}
