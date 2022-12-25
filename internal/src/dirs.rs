use super::Fuzzer;
use cargo_metadata::MetadataCommand;
use heck::ToSnakeCase;
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
pub fn crashes_directory_from_target(fuzzer: Fuzzer, krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(fuzzer, krate, target).join(match fuzzer {
        Fuzzer::Aflplusplus | Fuzzer::AflplusplusPersistent => "default/crashes",
        Fuzzer::Libfuzzer => "artifacts",
    })
}

#[must_use]
pub fn hangs_directory_from_target(fuzzer: Fuzzer, krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(fuzzer, krate, target).join(match fuzzer {
        Fuzzer::Aflplusplus | Fuzzer::AflplusplusPersistent => "default/hangs",
        Fuzzer::Libfuzzer => "artifacts",
    })
}

#[must_use]
pub fn queue_directory_from_target(fuzzer: Fuzzer, krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(fuzzer, krate, target).join(match fuzzer {
        Fuzzer::Aflplusplus | Fuzzer::AflplusplusPersistent => "default/queue",
        Fuzzer::Libfuzzer => "queue",
    })
}

#[must_use]
pub fn output_directory_from_target(fuzzer: Fuzzer, krate: &str, target: &str) -> PathBuf {
    output_directory(fuzzer).join(path_from_target(krate, target))
}

#[must_use]
fn impl_concretizations_directory() -> PathBuf {
    target_directory(None).join("impl_concretizations")
}

#[must_use]
fn concretizations_directory() -> PathBuf {
    target_directory(None).join("concretizations")
}

#[must_use]
fn corpus_directory() -> PathBuf {
    target_directory(None).join("corpus")
}

#[must_use]
fn output_directory(fuzzer: Fuzzer) -> PathBuf {
    target_directory(Some(fuzzer)).join("output")
}

#[must_use]
pub fn target_directory(fuzzer: Option<Fuzzer>) -> PathBuf {
    let mut command = MetadataCommand::new();
    if let Ok(path) = env::var("TEST_FUZZ_MANIFEST_PATH") {
        command.manifest_path(path);
    }
    let mut target_dir = command.no_deps().exec().unwrap().target_directory;
    if let Some(fuzzer) = fuzzer {
        target_dir = target_dir.join(fuzzer.to_string().to_snake_case());
    }
    target_dir.into()
}

#[must_use]
fn path_from_args_type<T>() -> String {
    let type_name = type_name::<T>();
    let n = type_name
        .find("_fuzz")
        .unwrap_or_else(|| panic!("unexpected type name: `{type_name}`"));
    type_name[..n].to_owned()
}

#[must_use]
fn path_from_target(krate: &str, target: &str) -> String {
    krate.replace('-', "_") + "::" + target
}
