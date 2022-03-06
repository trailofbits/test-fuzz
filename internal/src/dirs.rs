use cargo_metadata::MetadataCommand;
use std::{any::type_name, env, path::PathBuf};

#[must_use]
pub fn impl_concretizations_directory_from_args_type<T>() -> PathBuf {
    impl_concretizations_directory().join(path_from_args_type::<T>())
}

#[must_use]
pub fn impl_concretizations_directory_from_target(krate: &str, target: &str) -> PathBuf {
    impl_concretizations_directory().join(path_from_target(krate, target))
}

#[must_use]
pub fn concretizations_directory_from_args_type<T>() -> PathBuf {
    concretizations_directory().join(path_from_args_type::<T>())
}

#[must_use]
pub fn concretizations_directory_from_target(krate: &str, target: &str) -> PathBuf {
    concretizations_directory().join(path_from_target(krate, target))
}

#[must_use]
pub fn corpus_directory_from_args_type<T>() -> PathBuf {
    corpus_directory().join(path_from_args_type::<T>())
}

#[must_use]
pub fn corpus_directory_from_target(krate: &str, target: &str) -> PathBuf {
    corpus_directory().join(path_from_target(krate, target))
}

#[must_use]
pub fn crashes_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target)
        .join("default")
        .join("crashes")
}

#[must_use]
pub fn hangs_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target)
        .join("default")
        .join("hangs")
}

#[must_use]
pub fn queue_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target)
        .join("default")
        .join("queue")
}

#[must_use]
pub fn output_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory().join(path_from_target(krate, target))
}

#[must_use]
fn impl_concretizations_directory() -> PathBuf {
    target_directory(false).join("impl_concretizations")
}

#[must_use]
fn concretizations_directory() -> PathBuf {
    target_directory(false).join("concretizations")
}

#[must_use]
fn corpus_directory() -> PathBuf {
    target_directory(false).join("corpus")
}

#[must_use]
fn output_directory() -> PathBuf {
    target_directory(true).join("output")
}

#[must_use]
pub fn target_directory(instrumented: bool) -> PathBuf {
    let mut command = MetadataCommand::new();
    if let Ok(path) = env::var("TEST_FUZZ_MANIFEST_PATH") {
        command.manifest_path(path);
    }
    let mut target_dir = command.exec().unwrap().target_directory;
    if instrumented {
        target_dir = target_dir.join("afl");
    }
    target_dir.into()
}

#[must_use]
fn path_from_args_type<T>() -> String {
    let type_name = type_name::<T>();
    let n = type_name
        .find("_fuzz")
        .unwrap_or_else(|| panic!("unexpected type name: `{}`", type_name));
    type_name[..n].to_owned()
}

#[must_use]
fn path_from_target(krate: &str, target: &str) -> String {
    krate.replace('-', "_") + "::" + target
}
