use cargo_metadata::MetadataCommand;
use std::{
    any::type_name,
    env,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

pub static IN_TEST: AtomicBool = AtomicBool::new(false);

#[must_use]
pub fn impl_generic_args_directory_from_args_type<T>() -> PathBuf {
    impl_generic_args_directory().join(path_from_args_type::<T>())
}

#[must_use]
pub fn impl_generic_args_directory_from_target(krate: &str, target: &str) -> PathBuf {
    impl_generic_args_directory().join(path_from_target(krate, target))
}

#[must_use]
pub fn generic_args_directory_from_args_type<T>() -> PathBuf {
    generic_args_directory().join(path_from_args_type::<T>())
}

#[must_use]
pub fn generic_args_directory_from_target(krate: &str, target: &str) -> PathBuf {
    generic_args_directory().join(path_from_target(krate, target))
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
    output_directory_from_target(krate, target).join("default/crashes")
}

#[must_use]
pub fn hangs_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target).join("default/hangs")
}

#[must_use]
pub fn queue_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory_from_target(krate, target).join("default/queue")
}

#[must_use]
pub fn output_directory_from_target(krate: &str, target: &str) -> PathBuf {
    output_directory().join(path_from_target(krate, target))
}

#[must_use]
fn impl_generic_args_directory() -> PathBuf {
    #[allow(clippy::disallowed_methods)]
    target_directory(false, false).join(path_segment("impl_generic_args"))
}

#[must_use]
fn generic_args_directory() -> PathBuf {
    #[allow(clippy::disallowed_methods)]
    target_directory(false, false).join(path_segment("generic_args"))
}

#[must_use]
fn corpus_directory() -> PathBuf {
    #[allow(clippy::disallowed_methods)]
    target_directory(false, false).join(path_segment("corpus"))
}

#[must_use]
fn output_directory() -> PathBuf {
    #[allow(clippy::disallowed_methods)]
    target_directory(false, true).join(path_segment("output"))
}

#[must_use]
pub fn path_segment(s: &str) -> String {
    let maybe_id = maybe_id();
    format!(
        "{s}{}{}",
        if maybe_id.is_some() { "_" } else { "" },
        maybe_id.unwrap_or_default()
    )
}

fn maybe_id() -> Option<String> {
    env::var("TEST_FUZZ_ID").ok().or_else(|| {
        if IN_TEST.load(Ordering::SeqCst) {
            Some(thread_id())
        } else {
            None
        }
    })
}

fn thread_id() -> String {
    format!("{:?}", std::thread::current().id()).replace(['(', ')'], "_")
}

#[must_use]
pub fn target_directory(coverage: bool, fuzzing: bool) -> PathBuf {
    assert!(!(coverage && fuzzing));
    let mut command = MetadataCommand::new();
    if let Ok(path) = env::var("TEST_FUZZ_MANIFEST_PATH") {
        command.manifest_path(path);
    }
    let mut target_dir = command.no_deps().exec().unwrap().target_directory;
    if coverage {
        target_dir = target_dir.join("coverage");
    }
    if fuzzing {
        target_dir = target_dir.join("afl");
    }
    target_dir.into()
}

#[must_use]
fn path_from_args_type<T>() -> String {
    let type_name = type_name::<T>();
    let n = type_name
        .find("_fuzz__")
        .unwrap_or_else(|| panic!("unexpected type name: `{type_name}`"));
    type_name[..n].to_owned()
}

#[must_use]
fn path_from_target(krate: &str, target: &str) -> String {
    krate.replace('-', "_") + "::" + target
}
