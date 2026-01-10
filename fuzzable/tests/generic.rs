#![cfg_attr(dylint_lib = "general", allow(crate_wide_allow))]
#![allow(clippy::disallowed_names)]

use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::LazyLock};

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

trait BazTrait {
    fn qux();
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Baz<T>(T);

#[test_fuzz::test_fuzz_impl]
impl<T> BazTrait for Baz<T> {
    #[test_fuzz::test_fuzz(impl_generic_args = "Bar")]
    fn qux() {}
}

trait Trait<T: FooBarTrait> {
    fn target(&self, x: &T);
    fn target_bound<U: BazTrait + Clone + Debug + Serialize>(&self, x: &T, y: &U);
    fn target_where_clause<U>(&self, x: &T, y: &U)
    where
        U: BazTrait + Clone + Debug + Serialize;
    fn target_only_generic_args<U>(&self, x: &T, y: &U);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Struct;

#[test_fuzz::test_fuzz_impl]
impl<T: FooBarTrait + Clone + Debug + Serialize> Trait<T> for Struct {
    // smoelius: The Rust docs (https://doc.rust-lang.org/std/fmt/trait.Debug.html#stability)
    // state:
    // Derived `Debug` formats are not stable, and so may change with future Rust versions.
    // So `x` should not be compared to a string constant.
    #[allow(clippy::uninlined_format_args)]
    #[test_fuzz::test_fuzz(impl_generic_args = "Bar")]
    fn target(&self, x: &T) {
        assert_ne!(
            format!("{:?}", x),
            format!("{:?}", Bar::B("qwerty".to_owned()))
        );
    }

    #[test_fuzz::test_fuzz(impl_generic_args = "Bar", generic_args = "Baz<Bar>")]
    fn target_bound<U: BazTrait + Clone + Debug + Serialize>(&self, x: &T, y: &U) {}

    #[test_fuzz::test_fuzz(impl_generic_args = "Bar", generic_args = "Baz<Bar>")]
    fn target_where_clause<U>(&self, x: &T, y: &U)
    where
        U: BazTrait + Clone + Debug + Serialize,
    {
    }

    #[test_fuzz::test_fuzz(only_generic_args)]
    fn target_only_generic_args<U>(&self, _: &T, _: &U) {}
}

static FOO_QWERTY: LazyLock<Foo> = LazyLock::new(|| Foo::A("qwerty".to_owned()));
static BAR_ASDFGH: LazyLock<Bar> = LazyLock::new(|| Bar::B("asdfgh".to_owned()));

#[test]
fn test_foo_qwerty() {
    Struct.target(&*FOO_QWERTY);
}

#[test]
fn test_bar_asdfgh() {
    Struct.target(&*BAR_ASDFGH);
}

#[test]
fn test_bound() {
    Struct.target_bound(&*FOO_QWERTY, &Baz(FOO_QWERTY.clone()));
    Struct.target_bound(&*BAR_ASDFGH, &Baz(BAR_ASDFGH.clone()));
}

#[test]
fn test_where_clause() {
    Struct.target_where_clause(&*FOO_QWERTY, &Baz(FOO_QWERTY.clone()));
    Struct.target_where_clause(&*BAR_ASDFGH, &Baz(BAR_ASDFGH.clone()));
}

#[test]
fn test_only_generic_args() {
    Struct.target_only_generic_args(&*FOO_QWERTY, &Baz(FOO_QWERTY.clone()));
    Struct.target_only_generic_args(&*BAR_ASDFGH, &Baz(BAR_ASDFGH.clone()));
}

mod receiverless_trait_function {
    use super::*;

    trait Trait<T: Clone + Serialize> {
        fn target(x: &T);
    }

    #[test_fuzz::test_fuzz_impl]
    impl<T: Clone + Serialize> Trait<T> for Struct {
        #[test_fuzz::test_fuzz(impl_generic_args = "Bar")]
        fn target(x: &T) {}
    }
}
