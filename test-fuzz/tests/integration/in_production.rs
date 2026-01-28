use internal::dirs::{corpus_directory_from_target, path_segment};
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{read_dir, remove_dir_all},
    path::{Component, PathBuf},
    process::Command,
    sync::Mutex,
};
use testing::LoggedAssert;

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn no_write() {
    test(false, 0);
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn write() {
    test(true, 1);
}

static MUTEX: Mutex<()> = Mutex::new(());

fn test(write: bool, n: usize) {
    let _lock = MUTEX.lock().unwrap();

    let thread_specific_corpus = corpus_directory_from_target("hello-world", "intermediary");

    // smoelius: HACK. `hello-world` writes to `target/corpus`, not, e.g.,
    // `target/corpus_ThreadId_3_`. For now, just replace `corpus_ThreadId_3_` with `corpus`.
    let corpus = thread_specific_corpus
        .components()
        .map(|component| {
            if component.as_os_str() == OsString::from(path_segment("corpus")) {
                Component::Normal(OsStr::new("corpus"))
            } else {
                component
            }
        })
        .collect::<PathBuf>();

    // smoelius: This call to `remove_dir_all` is protected by the mutex above.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(&corpus).unwrap_or_default();

    let mut command = Command::new("cargo");
    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    command.args([
        "run",
        "--manifest-path",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../fuzzable/Cargo.toml"),
        "--features",
        &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
    ]);

    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let mut envs = vec![("TEST_FUZZ_MANIFEST_PATH", env!("CARGO_MANIFEST_PATH"))];

    if write {
        envs.push(("TEST_FUZZ_WRITE", "1"));
    }

    command
        .current_dir("/tmp")
        .envs(envs)
        .logged_assert()
        .success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);

    #[cfg(target_os = "linux")]
    if write {
        linux::generate_coverage();
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::LoggedAssert;
    use snapbox::{Redactions, assert_data_eq};
    use std::{fs::read_to_string, path::Path};
    use testing::fuzzable::test_fuzz_all;

    pub fn generate_coverage() {
        #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
        let parent = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let expected_sorted_lcov_info =
            read_to_string(parent.join("snapshots/sorted_hello_world_lcov.info")).unwrap();

        let mut command = test_fuzz_all().unwrap();
        // smoelius: As mentioned above, `hello-world` does not write to, e.g.,
        // `target/corpus_ThreadId_3_`. So do not set `TEST_FUZZ_ID`.
        command.env_remove("TEST_FUZZ_ID");
        command.args(["--coverage=corpus", "intermediary", "--exact"]);
        command.logged_assert().success();

        let actual_lcov_info = read_to_string("lcov.info").unwrap();
        let mut lines = actual_lcov_info.lines().collect::<Vec<_>>();
        lines.sort();
        let sorted_actual_lcov_info = lines
            .into_iter()
            .map(|s| format!("{s}\n"))
            .collect::<String>();

        let mut redactions = Redactions::new();
        redactions.insert("[REPO]", parent).unwrap();
        assert_data_eq!(
            redactions.redact(&sorted_actual_lcov_info),
            expected_sorted_lcov_info
        );
    }
}
