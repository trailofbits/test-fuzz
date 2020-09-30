use cargo_metadata::MetadataCommand;
use std::{any::type_name, path::PathBuf};

pub fn corpus_directory_from_args_type<T>() -> PathBuf {
    corpus_directory().join(path_from_args_type::<T>())
}

pub fn corpus_directory_from_target(krate: &str, target: &str) -> PathBuf {
    corpus_directory().join(path_from_target(krate, target))
}

pub fn crashes_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target).join("crashes")
}

pub fn queue_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target).join("queue")
}

pub fn output_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory().join(path_from_target(krate, target))
}

fn corpus_directory() -> PathBuf {
    target_directory().join("corpus")
}

fn output_directory() -> PathBuf {
    target_directory().join("afl")
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
