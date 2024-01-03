use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;

#[cfg(any(serde_default, feature = "__serde_bincode"))]
const BYTE_LIMIT: u64 = 1024 * 1024 * 1024;

#[allow(clippy::vec_init_then_push)]
#[must_use]
pub fn as_feature() -> &'static str {
    let mut formats = vec![];

    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    formats.push("serde_bincode");

    #[cfg(feature = "__serde_cbor")]
    formats.push("serde_cbor");

    #[cfg(feature = "__serde_cbor4ii")]
    formats.push("serde_cbor4ii");

    assert!(
        formats.len() <= 1,
        "{}",
        "Multiple serde formats selected: {formats:?}"
    );

    formats.pop().expect("No serde format selected")
}

#[must_use]
pub const fn serializes_variant_names() -> bool {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    return false;

    #[cfg(feature = "__serde_cbor")]
    return true;

    #[cfg(feature = "__serde_cbor4ii")]
    return true;
}

pub fn serialize<T: Serialize>(args: &T) -> Vec<u8> {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    return {
        use bincode::Options;
        // smoelius: From
        // https://github.com/bincode-org/bincode/blob/c44b5e364e7084cdbabf9f94b63a3c7f32b8fb68/src/lib.rs#L102-L103 :
        // /// **Warning:** the default configuration used by [`bincode::serialize`] is not
        // /// the same as that used by the `DefaultOptions` struct. ...
        // The point is that `bincode::serialize(..)` and `bincode::options().serialize(..)` use
        // different encodings, even though the latter uses "default" options.
        bincode::options()
            .with_limit(BYTE_LIMIT)
            .serialize(args)
            .unwrap()
    };

    #[cfg(feature = "__serde_cbor")]
    return serde_cbor::to_vec(args).unwrap();

    #[cfg(feature = "__serde_cbor4ii")]
    return {
        let mut data = Vec::new();
        cbor4ii::serde::to_writer(&mut data, args).unwrap();
        data
    };
}

pub fn deserialize<T: DeserializeOwned, R: Read>(reader: R) -> Option<T> {
    #[cfg(any(serde_default, feature = "__serde_bincode"))]
    return {
        use bincode::Options;
        bincode::options()
            .with_limit(BYTE_LIMIT)
            .deserialize_from(reader)
            .ok()
    };

    #[cfg(feature = "__serde_cbor")]
    return serde_cbor::from_reader(reader).ok();

    #[cfg(feature = "__serde_cbor4ii")]
    return {
        let reader = std::io::BufReader::new(reader);
        cbor4ii::serde::from_reader(reader).ok()
    };
}
