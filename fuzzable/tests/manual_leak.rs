use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WrappedKey<'key> {
    #[serde(with = "test_fuzz::serde_ref")]
    key: &'key [u8; 32],
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
