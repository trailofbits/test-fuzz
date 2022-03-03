use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_ref<S, T>(x: &&T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    <T as Serialize>::serialize(*x, serializer)
}

fn deserialize_ref<'de, D, T>(deserializer: D) -> Result<&'static T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + std::fmt::Debug,
{
    let x = <T as Deserialize>::deserialize(deserializer)?;
    Ok(Box::leak(Box::new(x)))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WrappedKey<'key> {
    #[serde(serialize_with = "serialize_ref", deserialize_with = "deserialize_ref")]
    pub key: &'key [u8; 32],
}

#[test_fuzz::test_fuzz_impl]
impl<'key> WrappedKey<'key> {
    #[test_fuzz::test_fuzz]
    fn dump(&self) {
        println!("{:?}", self.key);
    }
}

#[test]
fn test_dump() {
    let key: [u8; 32] = [1; 32];
    let wrapped_key = WrappedKey { key: &key };
    wrapped_key.dump();
}
