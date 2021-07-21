use dirs::{
    concretizations_directory_from_args_type, corpus_directory_from_args_type,
    impl_concretizations_directory_from_args_type,
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

#[cfg(feature = "serde_bincode")]
const BYTE_LIMIT: u64 = 1024 * 1024 * 1024;

#[derive(Copy, Clone)]
pub enum SerdeFormat {
    #[cfg(feature = "serde_bincode")]
    Bincode,
    #[cfg(feature = "serde_cbor")]
    Cbor,
}

// smoelius: TryDebug and TryDefault use Nikolai Vazquez's trick from `impls`.
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

pub trait TryDefaultFallback<U> {
    fn default() -> Option<U>;
}

impl<T, U> TryDefaultFallback<U> for T {
    #[must_use]
    fn default() -> Option<U> {
        None
    }
}

pub struct TryDefault<'a, T>(pub &'a T);

impl<'a, T: Default> TryDefault<'a, T> {
    #[must_use]
    pub fn default() -> Option<T> {
        Some(T::default())
    }
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
    match serde_format {
        #[cfg(feature = "serde_bincode")]
        SerdeFormat::Bincode => {
            use bincode::Options;
            let data = bincode::options()
                .with_limit(BYTE_LIMIT)
                .serialize(args)
                .unwrap();
            write_data(&corpus, &data).unwrap();
        }
        #[cfg(feature = "serde_cbor")]
        SerdeFormat::Cbor => {
            let data = serde_cbor::to_vec(args).unwrap();
            write_data(&corpus, &data).unwrap();
        }
    };
}

pub fn write_data(dir: &Path, data: &[u8]) -> io::Result<()> {
    create_dir_all(&dir).unwrap_or_default();
    let hex = {
        let hash = Sha1::digest(data);
        hex::encode(hash)
    };
    let path = dir.join(hex);
    write(path, &data)
}

#[allow(unused_variables)]
pub fn read_args<T: DeserializeOwned, R: Read>(serde_format: SerdeFormat, reader: R) -> Option<T> {
    match serde_format {
        #[cfg(feature = "serde_bincode")]
        SerdeFormat::Bincode => {
            use bincode::Options;
            bincode::options()
                .with_limit(BYTE_LIMIT)
                .deserialize_from(reader)
                .ok()
        }
        #[cfg(feature = "serde_cbor")]
        SerdeFormat::Cbor => serde_cbor::from_reader(reader).ok(),
    }
}
