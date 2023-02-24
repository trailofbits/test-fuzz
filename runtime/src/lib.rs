use internal::{
    dirs::{
        concretizations_directory_from_args_type, corpus_directory_from_args_type,
        impl_concretizations_directory_from_args_type,
    },
    SerdeFormat,
};
use serde::{de::DeserializeOwned, Serialize};
use sha1::{Digest, Sha1};
use std::{
    any::type_name,
    env,
    fmt::{self, Debug, Formatter},
    fs::{create_dir_all, write},
    io::{self, Read},
    marker::PhantomData,
    path::Path,
};

pub use num_traits;

pub mod traits;

#[cfg(any(serde_default, feature = "__serde_bincode"))]
const BYTE_LIMIT: u64 = 1024 * 1024 * 1024;

// smoelius: TryDebug, etc. use Nikolai Vazquez's trick from `impls`.
// https://github.com/nvzqz/impls#how-it-works

struct DebugUnimplemented<T>(PhantomData<T>);

impl<T> Debug for DebugUnimplemented<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const PAT: &str = "TryDebug<";
        let type_name = type_name::<T>();
        let pos = type_name.find(PAT).unwrap() + PAT.len();
        write!(
            f,
            "<unknown of type {}>",
            type_name[pos..].strip_suffix('>').unwrap()
        )
    }
}

pub trait TryDebugFallback {
    fn apply<U>(&self, f: &mut dyn FnMut(&dyn Debug) -> U) -> U;
}

impl<T> TryDebugFallback for T {
    fn apply<U>(&self, f: &mut dyn FnMut(&dyn Debug) -> U) -> U {
        f(&DebugUnimplemented::<Self>(PhantomData))
    }
}

pub struct TryDebug<'a, T>(pub &'a T);

impl<'a, T: Debug> TryDebug<'a, T> {
    pub fn apply<U>(&self, f: &mut dyn FnMut(&dyn Debug) -> U) -> U {
        f(self.0)
    }
}

/// * `ty` - Type. When `auto_impl!` is used in the `auto!` macro, this should be just `$ty`.
/// * `trait` - Trait that `ty` may or may not implement.
/// * `expr` - Vector of values using `trait`, should `ty` implement it. In `expr`, `T` should be
///   used as the name of the type, not `$ty`.
#[macro_export]
macro_rules! auto_impl {
    ($ty:ty, $trait:path, $expr:expr) => {{
        trait AutoFallback<U> {
            fn auto() -> Vec<U>;
        }

        impl<T, U> AutoFallback<U> for T {
            #[must_use]
            fn auto() -> Vec<U> {
                vec![]
            }
        }

        struct Auto<T>(std::marker::PhantomData<T>);

        impl<T: $trait> Auto<T> {
            #[must_use]
            pub fn auto() -> Vec<T> {
                $expr
            }
        }

        Auto::<$ty>::auto() as Vec<$ty>
    }};
}

#[macro_export]
macro_rules! auto {
    ($ty:ty) => {{
        let xss = [
            $crate::auto_impl!(
                $ty,
                $crate::num_traits::bounds::Bounded,
                vec![T::min_value(), T::max_value()]
            ),
            $crate::auto_impl!($ty, Default, vec![T::default()]),
            $crate::auto_impl!(
                $ty,
                $crate::traits::MaxValueSubOne,
                vec![T::max_value_sub_one()]
            ),
            $crate::auto_impl!($ty, $crate::traits::Middle, vec![T::low(), T::high()]),
            $crate::auto_impl!(
                $ty,
                $crate::traits::MinValueAddOne,
                vec![T::min_value_add_one()]
            ),
        ];
        IntoIterator::into_iter(xss).flatten()
    }};
}

#[must_use]
pub fn test_fuzz_enabled() -> bool {
    enabled("")
}

#[must_use]
pub fn display_enabled() -> bool {
    enabled("DISPLAY")
}

#[must_use]
pub fn pretty_print_enabled() -> bool {
    enabled("PRETTY_PRINT")
}

#[must_use]
pub fn replay_enabled() -> bool {
    enabled("REPLAY")
}

#[must_use]
pub fn write_enabled() -> bool {
    enabled("WRITE")
}

#[must_use]
fn enabled(opt: &str) -> bool {
    let key = "TEST_FUZZ".to_owned() + if opt.is_empty() { "" } else { "_" } + opt;
    env::var(key).map_or(false, |value| value != "0")
}

pub fn write_impl_concretization<T>(args: &[&str]) {
    let impl_concretizations = impl_concretizations_directory_from_args_type::<T>();
    let data = args.join(", ");
    write_data(&impl_concretizations, data.as_bytes()).unwrap();
}

pub fn write_concretization<T>(args: &[&str]) {
    let concretizations = concretizations_directory_from_args_type::<T>();
    let data = args.join(", ");
    write_data(&concretizations, data.as_bytes()).unwrap();
}

#[allow(unused_variables)]
pub fn write_args<T: Serialize>(serde_format: SerdeFormat, args: &T) {
    let corpus = corpus_directory_from_args_type::<T>();
    let data = match serde_format {
        #[cfg(any(serde_default, feature = "__serde_bincode"))]
        SerdeFormat::Bincode => {
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
        }
        #[cfg(feature = "__serde_cbor")]
        SerdeFormat::Cbor => serde_cbor::to_vec(args).unwrap(),
        #[cfg(feature = "__serde_cbor4ii")]
        SerdeFormat::Cbor4ii => {
            let mut data = Vec::new();
            cbor4ii::serde::to_writer(&mut data, args).unwrap();
            data
        }
    };
    write_data(&corpus, &data).unwrap();
}

pub fn write_data(dir: &Path, data: &[u8]) -> io::Result<()> {
    create_dir_all(dir).unwrap_or_default();
    let hex = {
        let hash = Sha1::digest(data);
        hex::encode(hash)
    };
    let path_buf = dir.join(hex);
    write(path_buf, data)
}

pub fn read_args<T: DeserializeOwned, R: Read>(serde_format: SerdeFormat, reader: R) -> Option<T> {
    match serde_format {
        #[cfg(any(serde_default, feature = "__serde_bincode"))]
        SerdeFormat::Bincode => {
            use bincode::Options;
            bincode::options()
                .with_limit(BYTE_LIMIT)
                .deserialize_from(reader)
                .ok()
        }
        #[cfg(feature = "__serde_cbor")]
        SerdeFormat::Cbor => serde_cbor::from_reader(reader).ok(),
        #[cfg(feature = "__serde_cbor4ii")]
        SerdeFormat::Cbor4ii => {
            let reader = std::io::BufReader::new(reader);
            cbor4ii::serde::from_reader(reader).ok()
        }
    }
}
