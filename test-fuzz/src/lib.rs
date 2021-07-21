pub use runtime;
pub use test_fuzz_macro::{test_fuzz, test_fuzz_impl};

// smoelius: Re-export afl so that test-fuzz clients do not need to add it to their Cargo.toml
// files.
#[cfg(feature = "persistent")]
pub use afl;

// smoelius: Unfortunately, the same trick doesn't work for serde.
// https://github.com/serde-rs/serde/issues/1465

#[allow(clippy::vec_init_then_push)]
#[must_use]
pub fn serde_format() -> &'static str {
    let mut formats = vec![];
    #[cfg(feature = "serde_bincode")]
    formats.push("bincode");
    #[cfg(feature = "serde_cbor")]
    formats.push("cbor");
    assert!(
        formats.len() <= 1,
        "Multiple serde formats selected: {:?}",
        formats
    );
    formats.pop().expect("No serde format selected")
}
