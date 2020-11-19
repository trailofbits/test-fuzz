use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
struct Foo;

struct Bar(Foo);

#[test_fuzz::test_fuzz_impl]
impl From<&Foo> for Bar {
    #[test_fuzz::test_fuzz]
    fn from(foo: &Foo) -> Bar {
        Bar(foo.clone())
    }
}

#[test]
fn test() {
    let _: Bar = (&Foo).into();
}
