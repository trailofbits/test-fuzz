pub trait FromRef<T> {
    fn from_ref(_: &T) -> Self;
}

impl<T, U> FromRef<T> for U
where
    T: Clone,
    U: From<T>,
{
    fn from_ref(value: &T) -> Self {
        Self::from(value.clone())
    }
}

/// Trait whose implementation is required by the [`test_fuzz` macro]'s [`convert`] option.
///
/// The reason for using a non-standard trait is to avoid conflicts that could arise from blanket
/// implementations of standard traits. For example, trying to use `From` instead of
/// `test_fuzz::Into` can lead to errors like the following:
///
/// ```text
/// conflicting implementation in crate `core`:
/// - impl<T> From<T> for T;
/// ```
///
/// Such errors were observed in the [Substrate Node Template] [third-party test].
///
/// [`convert`]: https://github.com/trailofbits/test-fuzz/blob/master/README.md#convert--x-y
/// [`test_fuzz` macro]: https://github.com/trailofbits/test-fuzz/blob/master/README.md#test_fuzz-macro
/// [Substrate Node Template]: https://github.com/trailofbits/test-fuzz/blob/master/cargo-test-fuzz/patches/substrate_node_template.patch
/// [third-party test]: https://github.com/trailofbits/test-fuzz/blob/master/cargo-test-fuzz/tests/third_party.rs
pub trait Into<T> {
    fn into(self) -> T;
}
