use cargo_metadata::MetadataCommand;
use std::{any::type_name, path::PathBuf};

pub fn corpus_directory_from_args_type<T>() -> PathBuf {
    corpus_directory().join(path_from_args_type::<T>())
}

pub fn corpus_directory_from_target(krate: &str, target: &str) -> PathBuf {
    corpus_directory().join(path_from_target(krate, target))
}

pub fn output_directory_from_target(krate: &str, target: &str) -> PathBuf {
    target_directory()
        .join("afl")
        .join(path_from_target(krate, target))
}

fn corpus_directory() -> PathBuf {
    target_directory().join("corpus")
}

fn target_directory() -> PathBuf {
    MetadataCommand::new().exec().unwrap().target_directory
}

fn path_from_args_type<T>() -> String {
    type_name::<T>()
        .strip_suffix("_fuzz::Args")
        .unwrap()
        .to_owned()
}

fn path_from_target(krate: &str, target: &str) -> String {
    krate.replace("-", "_") + "::" + target
}
