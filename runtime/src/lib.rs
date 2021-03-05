use crypto::{digest::Digest, sha1::Sha1};
use dirs::corpus_directory_from_args_type;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::type_name,
    env,
    fmt::{self, Debug, Formatter},
    fs::{create_dir_all, write},
    io::{self, Read},
    marker::PhantomData,
    path::Path,
};

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
            type_name[pos..].strip_suffix(">").unwrap()
        )
    }
}

pub trait TryDebugFallback {
    fn apply<U>(&self, f: &mut dyn FnMut(&dyn Debug) -> U) -> U;
}

impl<T> TryDebugFallback for T {
    fn apply<U>(&self, f: &mut dyn FnMut(&dyn Debug) -> U) -> U {
        f(&DebugUnimplemented::<T>(PhantomData))
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

pub fn write_args<T: Serialize>(args: &T) {
    let corpus = corpus_directory_from_args_type::<T>();
    let data = serde_cbor::to_vec(args).unwrap();
    write_data(&corpus, &data).unwrap();
}

pub fn write_data(corpus: &Path, data: &[u8]) -> io::Result<()> {
    create_dir_all(&corpus).unwrap_or_default();
    let hex = {
        let mut hasher = Sha1::new();
        hasher.input(&data);
        hasher.result_str()
    };
    let path = corpus.join(hex);
    write(path, &data)
}

pub fn read_args<T: DeserializeOwned, R: Read>(reader: R) -> Option<T> {
    serde_cbor::from_reader(reader).ok()
}
