use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
struct Foo;

#[derive(Clone)]
struct Bar<'a>(&'a Foo);

impl Serialize for Bar<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Bar<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <()>::deserialize(deserializer).map(|()| Bar(&Foo))
    }
}

#[test_fuzz::test_fuzz]
fn target<'a>(bar: Bar<'a>) {
    println!("{:?}", bar.0);
}

#[test]
fn test() {
    target(Bar(&Foo));
}
