#![cfg_attr(dylint_lib = "crate_wide_allow", allow(crate_wide_allow))]
#![allow(clippy::blacklisted_name)]
#![allow(clippy::from_over_into)]
#![allow(clippy::needless_arbitrary_self_type)]
#![allow(clippy::similar_names)]

use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Deserialize, Serialize)]
struct Foo;

#[derive(Clone, Deserialize, Serialize)]
struct Bar(Option<Foo>);

#[derive(Clone, Deserialize, Serialize)]
struct Baz(Foo);

#[test_fuzz::test_fuzz_impl]
impl From<Foo> for Bar {
    #[test_fuzz::test_fuzz]
    fn from(foo: Foo) -> Self {
        Self(Some(foo))
    }
}

#[test_fuzz::test_fuzz_impl]
impl TryFrom<Bar> for Baz {
    type Error = &'static str;
    #[test_fuzz::test_fuzz]
    fn try_from(bar: Bar) -> Result<Self, Self::Error> {
        bar.0.ok_or("None").map(Baz)
    }
}

#[test_fuzz::test_fuzz_impl]
impl Into<Foo> for Baz {
    #[test_fuzz::test_fuzz]
    fn into(self: Self) -> Foo {
        self.0
    }
}

#[test]
fn test() {
    let bar: Bar = Foo.into();
    let baz: Baz = bar.try_into().unwrap();
    let _: Foo = baz.into();
}
