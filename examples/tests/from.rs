#![allow(clippy::blacklisted_name)]

use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Deserialize, Serialize)]
struct Foo;

#[derive(Clone, Deserialize, Serialize)]
struct Bar(Option<Foo>);

#[test_fuzz::test_fuzz_impl]
impl From<Foo> for Bar {
    #[test_fuzz::test_fuzz]
    fn from(foo: Foo) -> Self {
        Bar(Some(foo))
    }
}

#[test_fuzz::test_fuzz_impl]
impl TryFrom<Bar> for Foo {
    type Error = &'static str;
    #[test_fuzz::test_fuzz]
    fn try_from(bar: Bar) -> Result<Self, Self::Error> {
        bar.0.ok_or("None")
    }
}

#[test]
fn test() {
    let bar: Bar = Foo.into();
    let _: Foo = bar.try_into().unwrap();
}
