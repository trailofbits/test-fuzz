use serde::{Deserialize, Deserializer, Serialize, Serializer};

struct Foo;

#[derive(Clone)]
struct Bar<'a>(&'a Foo);

impl<'a> Serialize for Bar<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ().serialize(serializer)
    }
}

impl<'de, 'a> Deserialize<'de> for Bar<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <()>::deserialize(deserializer).map(|_| Bar(&Foo))
    }
}

#[test_fuzz::test_fuzz]
fn target<'a>(bar: Bar<'a>) {}

#[test]
fn test() {
    target(Bar(&Foo));
}
