use crypto::{digest::Digest, sha1::Sha1};
use dirs::corpus_directory_from_args_type;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    env,
    fs::{create_dir_all, write},
    io::Read,
};

pub fn fuzzing() -> bool {
    env::var("TEST_FUZZ").map_or(false, |s| !s.is_empty())
}

pub fn write_args<T: Serialize>(args: &T) {
    let corpus = corpus_directory_from_args_type::<T>();
    create_dir_all(&corpus).unwrap_or_default();

    let data = serde_cbor::to_vec(args).unwrap();
    let hex = {
        let mut hasher = Sha1::new();
        hasher.input(&data);
        hasher.result_str()
    };
    let path = corpus.join(hex);
    write(path, &data).unwrap();
}

pub fn read_args<T: DeserializeOwned, R: Read>(reader: R) -> Option<T> {
    serde_cbor::from_reader(reader).ok()
}
