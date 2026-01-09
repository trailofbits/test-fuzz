use internal::{
    dirs::{
        corpus_directory_from_args_type, generic_args_directory_from_args_type,
        impl_generic_args_directory_from_args_type,
    },
    serde_format,
};
use serde::{Serialize, de::DeserializeOwned};
use sha1::{Digest, Sha1};
use std::{
    any::type_name,
    env,
    fmt::{self, Debug, Formatter},
    fs::{create_dir_all, write},
    io::{self, Read, Write},
    marker::PhantomData,
    path::Path,
    sync::Once,
};

pub use num_traits;

pub mod traits;

// smoelius: TryDebug, etc. use Nikolai Vazquez's trick from `impls`.
// https://github.com/nvzqz/impls#how-it-works

struct DebugUnimplemented<T>(PhantomData<T>);

impl<T> Debug for DebugUnimplemented<T> {
    // smoelius: The third party-tests check for the absence of "unknown of type..." messages. So if
    // the messages change, the third-party tests must be updated.
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

impl<T: Debug> TryDebug<'_, T> {
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

pub fn warn_if_test_fuzz_not_enabled() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if !test_fuzz_enabled() {
            #[allow(clippy::explicit_write)]
            writeln!(
                io::stderr(),
                "If you are trying to run a test-fuzz-generated fuzzing harness, be sure to run \
                 with `TEST_FUZZ=1`."
            )
            .unwrap();
        }
    });
}

#[must_use]
pub fn test_fuzz_enabled() -> bool {
    enabled("")
}

#[must_use]
pub fn coverage_enabled() -> bool {
    enabled("COVERAGE")
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
    env::var(key).is_ok_and(|value| value != "0")
}

pub fn write_impl_generic_args<T>(args: &[&str]) {
    let impl_generic_args = impl_generic_args_directory_from_args_type::<T>();
    let data = args.join(", ");
    write_data(&impl_generic_args, data.as_bytes()).unwrap();
}

pub fn write_generic_args<T>(args: &[&str]) {
    let generic_args = generic_args_directory_from_args_type::<T>();
    let data = args.join(", ");
    write_data(&generic_args, data.as_bytes()).unwrap();
}

pub fn write_args<T: Serialize>(args: &T) {
    let corpus = corpus_directory_from_args_type::<T>();
    let data = serde_format::serialize(args);
    write_data(&corpus, &data).unwrap();
}

pub fn write_data(dir: &Path, data: &[u8]) -> io::Result<()> {
    create_dir_all(dir).unwrap_or_default();
    let hex = {
        let digest = Sha1::digest(data);
        hex::encode(digest)
    };
    let path_buf = dir.join(hex);
    write(path_buf, data)
}

pub fn read_args<T: DeserializeOwned, R: Read>(reader: R) -> Option<T> {
    serde_format::deserialize(reader)
}
