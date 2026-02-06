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
        linux::generate_and_check_coverage();
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::LoggedAssert;
    use lcov::{Reader, Record};
    use regex::Regex;
    use rustc_demangle::demangle;
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs::read_to_string,
        os::unix::ffi::OsStrExt,
        path::Path,
        sync::LazyLock,
    };
    use testing::fuzzable::test_fuzz_all;

    type FunctionCoverage = BTreeMap<String, u64>;

    type LineCoverage = BTreeMap<String, BTreeSet<u32>>;

    static EXPECTED_FUNCTION_COVERAGE: LazyLock<FunctionCoverage> = LazyLock::new(|| {
        [
            (String::from("hello_world::main"), 0),
            (String::from("hello_world::target"), 1),
        ]
        .into_iter()
        .collect()
    });

    #[cfg_attr(dylint_lib = "supplementary", allow(unnamed_constant))]
    static EXPECTED_LINE_COVERAGE: LazyLock<LineCoverage> = LazyLock::new(|| {
        std::iter::once((
            String::from("fuzzable/src/bin/hello-world.rs"),
            [10, 11, 12].into_iter().collect(),
        ))
        .collect()
    });

    pub fn generate_and_check_coverage() {
        #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
        let parent = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

        let mut command = test_fuzz_all().unwrap();
        // smoelius: As mentioned above, `hello-world` does not write to, e.g.,
        // `target/corpus_ThreadId_3_`. So do not set `TEST_FUZZ_ID`.
        command.env_remove("TEST_FUZZ_ID");
        command.args(["--coverage=corpus", "intermediary", "--exact"]);
        command.logged_assert().success();

        let lcov_info = read_to_string("lcov.info").unwrap();
        let (function_coverage, line_coverage) = lcov_demangle(&lcov_info);

        let hello_world_function_coverage = function_coverage
            .into_iter()
            .filter(|(name, _)| name.starts_with("hello_world::"))
            .collect::<FunctionCoverage>();
        assert_eq!(*EXPECTED_FUNCTION_COVERAGE, hello_world_function_coverage);

        let hello_world_line_coverage = line_coverage
            .into_iter()
            .filter_map(|(path, count)| {
                let stripped = Path::new(&path).strip_prefix(parent).ok()?;
                if stripped.starts_with("fuzzable") {
                    Some((stripped.to_string_lossy().to_string(), count))
                } else {
                    None
                }
            })
            .collect::<LineCoverage>();
        assert_eq!(*EXPECTED_LINE_COVERAGE, hello_world_line_coverage);
    }

    // smoelius: `lcov_demangle` is loosely based on `lcov_read` from `cargo-line-test`:
    // https://github.com/trailofbits/cargo-line-test/blob/ebe2fb110eaeb2255d919c838edcc078a0147467/src/db/read.rs#L97
    fn lcov_demangle(lcov: &str) -> (FunctionCoverage, LineCoverage) {
        let mut function_coverage = FunctionCoverage::default();
        let mut line_coverage = LineCoverage::default();
        let mut source_file = None;
        for result in Reader::new(lcov.as_bytes()) {
            match result.unwrap() {
                Record::SourceFile { path } => {
                    if let Some(source_file) = &source_file {
                        panic!("source file already given: {source_file}");
                    }
                    let path_utf8 = std::str::from_utf8(path.as_os_str().as_bytes()).unwrap();
                    source_file = Some(path_utf8.to_owned());
                }
                Record::LineData {
                    line,
                    count,
                    checksum: _,
                } if count != 0 => {
                    let Some(source_file) = &source_file else {
                        panic!("source file not given");
                    };
                    line_coverage
                        .entry(source_file.clone())
                        .or_default()
                        .insert(line);
                }
                Record::FunctionData { name, count } => {
                    let demangled_name = demangle(&name).to_string();
                    let stripped_demangled_name = strip_disambiguators(&demangled_name);
                    let existing_count = function_coverage
                        .entry(stripped_demangled_name)
                        .or_default();
                    *existing_count += count;
                }
                Record::EndOfRecord => {
                    source_file = None;
                }
                _ => {}
            }
        }
        (function_coverage, line_coverage)
    }

    static DISAMBIGUATOR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[[[:xdigit:]]*\]").unwrap());

    fn strip_disambiguators(name: &str) -> String {
        DISAMBIGUATOR_RE.replace_all(name, "").to_string()
    }
}
