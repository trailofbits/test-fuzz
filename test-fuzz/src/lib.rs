pub use runtime;
pub use test_fuzz_macro::{test_fuzz, test_fuzz_impl};

// smoelius: Re-export afl so that test-fuzz clients do not need to add it to their Cargo.toml
// files.
#[cfg(feature = "__persistent")]
pub use afl;

// smoelius: Unfortunately, the same trick doesn't work for serde.
// https://github.com/serde-rs/serde/issues/1465

pub use internal::{serde_format, SerdeFormat};

mod utils;
pub use utils::{deserialize_ref, serialize_ref};

mod convert;
pub use convert::{FromRef, Into};
