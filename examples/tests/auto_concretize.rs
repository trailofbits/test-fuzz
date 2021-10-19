#![cfg(feature = "__auto_concretize")]

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
struct Foo<T>(T);

#[test_fuzz::test_fuzz_impl]
impl<T: Clone + DeserializeOwned + Serialize> Foo<T> {
    #[test_fuzz::test_fuzz]
    fn target<U: Clone + DeserializeOwned + Serialize>(x: &U) {}
}

#[derive(Clone, Deserialize, Serialize)]
struct Bar;

#[derive(Clone, Deserialize, Serialize)]
struct Baz;

#[test]
fn test() {
    Foo::<Bar>::target(&Baz)
}
