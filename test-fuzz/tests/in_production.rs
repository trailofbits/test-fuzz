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

#[test]
fn no_write() {
    test(false, 0);
}

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
    static ref LOCK: Mutex<()> = Mutex::new(());
}

fn test(write: bool, n: usize) {
    let _guard = LOCK.lock().unwrap();

    let corpus = corpus_directory_from_target("hello-world", "target");

    remove_dir_all(&corpus).unwrap_or_default();

    let mut command = Command::cargo_bin("hello-world").unwrap();

    let mut envs = vec![("TEST_FUZZ_MANIFEST_PATH", MANIFEST_PATH.as_str())];

    if write {
        envs.push(("TEST_FUZZ_WRITE", "1"));
    }

    command.current_dir("/tmp").envs(envs).assert().success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
