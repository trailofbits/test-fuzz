pub mod dirs;
pub mod examples;

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
