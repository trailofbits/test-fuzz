use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;

#[cfg(any(serde_default, feature = "__serde_bincode"))]
const BYTE_LIMIT: usize = 1024 * 1024 * 1024;

// smoelius: I can't find any guidance on how to choose this size. 2048 is used in the `loopback`
// test in the `postcard` repository:
// https://github.com/jamesmunns/postcard/blob/03865c2b7d694d000c0457e8cfaf4ff1b128ed81/tests/loopback.rs#L191
#[cfg(feature = "__serde_postcard")]
const SLIDING_BUFFER_SIZE: usize = 2048;

#[allow(clippy::vec_init_then_push)]
#[must_use]
pub fn as_feature() -> &'static str {
    let mut formats = vec![];

    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    formats.push("serde_bincode");

    #[cfg(feature = "__serde_postcard")]
    formats.push("serde_postcard");

    assert!(
        formats.len() <= 1,
        "Multiple serde formats selected: {formats:?}"
    );

    formats.pop().expect("No serde format selected")
}

pub fn serialize<T: Serialize>(args: &T) -> Vec<u8> {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    return {
        // smoelius: From
        // https://github.com/bincode-org/bincode/blob/c44b5e364e7084cdbabf9f94b63a3c7f32b8fb68/src/lib.rs#L102-L103 :
        // /// **Warning:** the default configuration used by [`bincode::serialize`] is not
        // /// the same as that used by the `DefaultOptions` struct. ...
        // The point is that `bincode::serialize(..)` and `bincode::options().serialize(..)` use
        // different encodings, even though the latter uses "default" options.
        // smoelius: With the upgrade to Bincode 2.0, the preceding comments may no longer be
        // applicable.
        let config = bincode::config::standard().with_limit::<BYTE_LIMIT>();
        bincode::serde::encode_to_vec(args, config).unwrap()
    };

    #[cfg(feature = "__serde_postcard")]
    return {
        let mut data = Vec::new();
        postcard::to_io(args, &mut data).unwrap();
        data
    };
}

pub fn deserialize<T: DeserializeOwned, R: Read>(mut reader: R) -> Option<T> {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    return {
        let config = bincode::config::standard().with_limit::<BYTE_LIMIT>();
        bincode::serde::decode_from_std_read(&mut reader, config).ok()
    };

    #[cfg(feature = "__serde_postcard")]
    return {
        let mut buff = [0; SLIDING_BUFFER_SIZE];
        postcard::from_io((reader, &mut buff))
            .map(|(value, _)| value)
            .ok()
    };
}
