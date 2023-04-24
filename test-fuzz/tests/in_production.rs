use assert_cmd::prelude::*;
use internal::dirs::corpus_directory_from_target;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{read_dir, remove_dir_all},
    path::Path,
    process::Command,
    sync::Mutex,
};

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn no_write() {
    test(false, 0);
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn write() {
    test(true, 1);
}

#[cfg(test)]
lazy_static! {
    static ref MANIFEST_PATH: String = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("Cargo.toml")
        .to_string_lossy()
        .to_string();
}

static MUTEX: Mutex<()> = Mutex::new(());

fn test(write: bool, n: usize) {
    let _lock = MUTEX.lock().unwrap();

    let corpus = corpus_directory_from_target("hello-world", "target");

    // smoelius: This call to `remove_dir_all` is protected by the mutex above.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    let mut command = Command::cargo_bin("hello-world").unwrap();

    let mut envs = vec![("TEST_FUZZ_MANIFEST_PATH", MANIFEST_PATH.as_str())];

    if write {
        envs.push(("TEST_FUZZ_WRITE", "1"));
    }

    command.current_dir("/tmp").envs(envs).assert().success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
