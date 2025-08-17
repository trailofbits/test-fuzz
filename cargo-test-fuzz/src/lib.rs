//! # cargo-test-fuzz
//!
//! This crate provides the implementation for the `cargo test-fuzz` subcommand.
//!
//! ## Primary Exports
//!
//! - [`run`](fn.run.html): The main entry point function for executing fuzzing operations
//! - [`TestFuzz`](struct.TestFuzz.html): Configuration struct containing all fuzzing options
//! - [`Object`](enum.Object.html): Enum representing different types of fuzzing artifacts
//!
//! For more information on `test-fuzz`, see the project's [README].
//!
//! [README]: https://github.com/trailofbits/test-fuzz/blob/master/README.md

#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::panic)]

use anyhow::{anyhow, bail, ensure, Context, Result};
use bitflags::bitflags;
use cargo_metadata::{
    Artifact, ArtifactProfile, CargoOpt, Message, Metadata, MetadataCommand, Package, PackageId,
};
use clap::{crate_version, ValueEnum};
use heck::ToKebabCase;
use internal::dirs::{
    corpus_directory_from_target, crashes_directory_from_target,
    generic_args_directory_from_target, hangs_directory_from_target,
    impl_generic_args_directory_from_target, output_directory_from_target,
    queue_directory_from_target, target_directory,
};
use log::debug;
use mio::{unix::pipe::Receiver, Events, Interest, Poll, Token};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::{Debug, Formatter},
    fs::{create_dir_all, read, read_dir, remove_dir_all, File},
    io::{BufRead, Read},
    iter,
    path::{Path, PathBuf},
    process::{exit, Child as StdChild, Command, Stdio},
    sync::OnceLock,
    time::Duration,
};
use strum_macros::Display;
use subprocess::{CommunicateError, Exec, ExitStatus, NullFile, Redirection};

const AUTO_GENERATED_SUFFIX: &str = "_fuzz__::auto_generate";
const ENTRY_SUFFIX: &str = "_fuzz__::entry";

const BASE_ENVS: &[(&str, &str)] = &[("TEST_FUZZ", "1"), ("TEST_FUZZ_WRITE", "0")];

const DEFAULT_TIMEOUT: u64 = 1;

const MILLIS_PER_SEC: u64 = 1_000;

bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq)]
    struct Flags: u8 {
        const REQUIRES_CARGO_TEST = 0b0000_0001;
        const RAW = 0b0000_0010;
    }
}

#[derive(Clone, Copy, Debug, Display, Deserialize, PartialEq, Eq, Serialize, ValueEnum)]
#[remain::sorted]
pub enum Object {
    Corpus,
    CorpusInstrumented,
    Crashes,
    CrashesInstrumented,
    GenericArgs,
    Hangs,
    HangsInstrumented,
    ImplGenericArgs,
    Queue,
    QueueInstrumented,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[remain::sorted]
pub struct TestFuzz {
    pub backtrace: bool,
    pub consolidate: bool,
    pub consolidate_all: bool,
    pub cpus: Option<usize>,
    pub display: Option<Object>,
    pub exact: bool,
    pub exit_code: bool,
    pub features: Vec<String>,
    pub list: bool,
    pub manifest_path: Option<String>,
    pub max_total_time: Option<u64>,
    pub no_default_features: bool,
    pub no_run: bool,
    pub no_ui: bool,
    pub package: Option<String>,
    pub persistent: bool,
    pub pretty: bool,
    pub release: bool,
    pub replay: Option<Object>,
    pub reset: bool,
    pub reset_all: bool,
    pub resume: bool,
    pub run_until_crash: bool,
    pub slice: u64,
    pub test: Option<String>,
    pub timeout: Option<u64>,
    pub verbose: bool,
    pub ztarget: Option<String>,
    pub zzargs: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Executable {
    path: PathBuf,
    name: String,
    test_fuzz_version: Option<Version>,
    afl_version: Option<Version>,
}

impl Debug for Executable {
    #[cfg_attr(dylint_lib = "general", allow(non_local_effect_before_error_return))]
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let test_fuzz_version = self
            .test_fuzz_version
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let afl_version = self
            .afl_version
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        fmt.debug_struct("Executable")
            .field("path", &self.path)
            .field("name", &self.name)
            .field("test_fuzz_version", &test_fuzz_version)
            .field("afl_version", &afl_version)
            .finish()
    }
}

pub fn run(opts: TestFuzz) -> Result<()> {
    let opts = {
        let mut opts = opts;
        if opts.exit_code {
            opts.no_ui = true;
        }
        opts
    };

    let no_instrumentation = opts.list
        || matches!(
            opts.display,
            Some(
                Object::Corpus
                    | Object::Crashes
                    | Object::Hangs
                    | Object::ImplGenericArgs
                    | Object::GenericArgs
                    | Object::Queue
            )
        )
        || matches!(
            opts.replay,
            Some(Object::Corpus | Object::Crashes | Object::Hangs | Object::Queue)
        );

    run_without_exit_code(&opts, !no_instrumentation).map_err(|error| {
        if opts.exit_code {
            eprintln!("{error:?}");
            exit(2);
        }
        error
    })
}

#[doc(hidden)]
pub fn run_without_exit_code(opts: &TestFuzz, use_instrumentation: bool) -> Result<()> {
    if let Some(object) = opts.replay {
        ensure!(
            !matches!(object, Object::ImplGenericArgs | Object::GenericArgs),
            "`--replay {}` is invalid.",
            object.to_string().to_kebab_case()
        );
    }

    // smoelius: Ensure `cargo-afl` is installed.
    let _ = cached_cargo_afl_version();

    let display = opts.display.is_some();

    let replay = opts.replay.is_some();

    let executables = build(opts, use_instrumentation, display || replay)?;

    let mut executable_targets = executable_targets(&executables)?;

    if let Some(pat) = &opts.ztarget {
        executable_targets = filter_executable_targets(opts, pat, &executable_targets);
    }

    check_test_fuzz_and_afl_versions(&executable_targets)?;

    if opts.list {
        println!("{executable_targets:#?}");
        return Ok(());
    }

    if opts.no_run {
        return Ok(());
    }

    if opts.consolidate_all || opts.reset_all {
        if opts.consolidate_all {
            consolidate(opts, &executable_targets)?;
        }
        return reset(opts, &executable_targets);
    }

    if opts.consolidate || opts.reset || display || replay {
        let (executable, target) = executable_target(opts, &executable_targets)?;

        if opts.consolidate || opts.reset {
            if opts.consolidate {
                consolidate(opts, &executable_targets)?;
            }
            return reset(opts, &executable_targets);
        }

        let (flags, dir) = None
            .or_else(|| {
                opts.display
                    .map(|object| flags_and_dir(object, &executable.name, &target))
            })
            .or_else(|| {
                opts.replay
                    .map(|object| flags_and_dir(object, &executable.name, &target))
            })
            .unwrap_or_else(|| (Flags::empty(), PathBuf::default()));

        return for_each_entry(opts, &executable, &target, display, replay, flags, &dir);
    }

    if !use_instrumentation {
        return Ok(());
    }

    let executable_targets = flatten_executable_targets(opts, executable_targets)?;

    fuzz(opts, &executable_targets)
}

#[allow(clippy::too_many_lines)]
fn build(opts: &TestFuzz, use_instrumentation: bool, quiet: bool) -> Result<Vec<Executable>> {
    let metadata = metadata(opts)?;
    let silence_stderr = quiet && !opts.verbose;

    let mut args = vec![];
    if use_instrumentation {
        args.extend_from_slice(&["afl"]);
    }
    args.extend_from_slice(&["test", "--offline", "--no-run"]);
    if opts.no_default_features {
        args.extend_from_slice(&["--no-default-features"]);
    }
    for features in &opts.features {
        args.extend_from_slice(&["--features", features]);
    }
    if opts.release {
        args.extend_from_slice(&["--release"]);
    }
    let target_dir = target_directory(true);
    let target_dir_str = target_dir.to_string_lossy();
    if use_instrumentation {
        args.extend_from_slice(&["--target-dir", &target_dir_str]);
    }
    if let Some(path) = &opts.manifest_path {
        args.extend_from_slice(&["--manifest-path", path]);
    }
    if let Some(package) = &opts.package {
        args.extend_from_slice(&["--package", package]);
    }
    if opts.persistent {
        args.extend_from_slice(&["--features", "test-fuzz/__persistent"]);
    }
    if let Some(name) = &opts.test {
        args.extend_from_slice(&["--test", name]);
    }

    // smoelius: Suppress "Warning: AFL++ tools will need to set AFL_MAP_SIZE..." Setting
    // `AFL_QUIET=1` doesn't work here, so pipe standard error to: /dev/null
    // smoelius: Suppressing all of standard error is too extreme. For now, suppress only when
    // displaying/replaying.
    let mut exec = Exec::cmd("cargo")
        .args(
            &args
                .iter()
                .chain(iter::once(&"--message-format=json"))
                .collect::<Vec<_>>(),
        )
        .stdout(Redirection::Pipe);
    if silence_stderr {
        exec = exec.stderr(NullFile);
    }
    debug!("{exec:?}");
    let mut popen = exec.clone().popen()?;
    let messages = popen
        .stdout
        .take()
        .map_or(Ok(vec![]), |stream| -> Result<_> {
            let reader = std::io::BufReader::new(stream);
            Message::parse_stream(reader)
                .collect::<std::result::Result<_, std::io::Error>>()
                .with_context(|| format!("`parse_stream` failed for `{exec:?}`"))
        })?;
    let status = popen
        .wait()
        .with_context(|| format!("`wait` failed for `{popen:?}`"))?;

    if !status.success() {
        // smoelius: If stderr was silenced, re-execute the command without --message-format=json.
        // This is easier than trying to capture and colorize `CompilerMessage`s like Cargo does.
        // smoelius: Rather than re-execute the command, just debug print the messages.
        eprintln!("{messages:#?}");
        bail!("Command failed: {:?}", exec);
    }

    let executables = messages
        .into_iter()
        .map(|message| {
            if let Message::CompilerArtifact(Artifact {
                package_id,
                target: build_target,
                profile: ArtifactProfile { test: true, .. },
                executable: Some(executable),
                ..
            }) = message
            {
                let (test_fuzz_version, afl_version) =
                    test_fuzz_and_afl_versions(&metadata, &package_id)?;
                Ok(Some(Executable {
                    path: executable.into(),
                    name: build_target.name,
                    test_fuzz_version,
                    afl_version,
                }))
            } else {
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(executables.into_iter().flatten().collect())
}

fn metadata(opts: &TestFuzz) -> Result<Metadata> {
    let mut command = MetadataCommand::new();
    if opts.no_default_features {
        command.features(CargoOpt::NoDefaultFeatures);
    }
    let mut features = opts.features.clone();
    features.push("test-fuzz/__persistent".to_owned());
    command.features(CargoOpt::SomeFeatures(features));
    if let Some(path) = &opts.manifest_path {
        command.manifest_path(path);
    }
    command.exec().map_err(Into::into)
}

fn test_fuzz_and_afl_versions(
    metadata: &Metadata,
    package_id: &PackageId,
) -> Result<(Option<Version>, Option<Version>)> {
    let test_fuzz = package_dependency(metadata, package_id, "test-fuzz")?;
    let afl = test_fuzz
        .as_ref()
        .map(|package_id| package_dependency(metadata, package_id, "afl"))
        .transpose()?;
    let test_fuzz_version = test_fuzz
        .map(|package_id| package_version(metadata, &package_id))
        .transpose()?;
    let afl_version = afl
        .flatten()
        .map(|package_id| package_version(metadata, &package_id))
        .transpose()?;
    Ok((test_fuzz_version, afl_version))
}

fn package_dependency(
    metadata: &Metadata,
    package_id: &PackageId,
    name: &str,
) -> Result<Option<PackageId>> {
    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| anyhow!("No dependency graph"))?;
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == *package_id)
        .ok_or_else(|| anyhow!("Could not find package `{}`", package_id))?;
    let package_ids_and_names = node
        .dependencies
        .iter()
        .map(|package_id| {
            package_name(metadata, package_id).map(|package_name| (package_id, package_name))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(package_ids_and_names
        .into_iter()
        .find_map(|(package_id, package_name)| {
            if package_name == name {
                Some(package_id.clone())
            } else {
                None
            }
        }))
}

fn package_name(metadata: &Metadata, package_id: &PackageId) -> Result<String> {
    package(metadata, package_id).map(|package| package.name.clone())
}

fn package_version(metadata: &Metadata, package_id: &PackageId) -> Result<Version> {
    package(metadata, package_id).map(|package| package.version.clone())
}

fn package<'a>(metadata: &'a Metadata, package_id: &PackageId) -> Result<&'a Package> {
    metadata
        .packages
        .iter()
        .find(|package| package.id == *package_id)
        .ok_or_else(|| anyhow!("Could not find package `{}`", package_id))
}

fn executable_targets(executables: &[Executable]) -> Result<Vec<(Executable, Vec<String>)>> {
    let executable_targets: Vec<(Executable, Vec<String>)> = executables
        .iter()
        .map(|executable| {
            let targets = targets(&executable.path)?;
            Ok((executable.clone(), targets))
        })
        .collect::<Result<_>>()?;

    Ok(executable_targets
        .into_iter()
        .filter(|(_, targets)| !targets.is_empty())
        .collect())
}

fn targets(executable: &Path) -> Result<Vec<String>> {
    let exec = Exec::cmd(executable)
        .env_extend(&[("AFL_QUIET", "1")])
        .args(&["--list", "--format=terse"])
        .stderr(NullFile);
    debug!("{exec:?}");
    let stream = exec.clone().stream_stdout()?;

    // smoelius: A test executable's --list output ends with an empty line followed by
    // "M tests, N benchmarks." Stop at the empty line.
    // smoelius: Searching for the empty line is not necessary: https://stackoverflow.com/a/64913357
    let mut targets = Vec::<String>::default();
    for line in std::io::BufReader::new(stream).lines() {
        let line = line.with_context(|| format!("Could not get output of `{exec:?}`"))?;
        let Some(line) = line.strip_suffix(": test") else {
            continue;
        };
        let Some(line) = line.strip_suffix(ENTRY_SUFFIX) else {
            continue;
        };
        targets.push(line.to_owned());
    }
    Ok(targets)
}

#[test_fuzz::test_fuzz]
fn filter_executable_targets(
    opts: &TestFuzz,
    pat: &str,
    executable_targets: &[(Executable, Vec<String>)],
) -> Vec<(Executable, Vec<String>)> {
    executable_targets
        .iter()
        .filter_map(|(executable, targets)| {
            let targets = filter_targets(opts, pat, targets);
            if targets.is_empty() {
                None
            } else {
                Some((executable.clone(), targets))
            }
        })
        .collect()
}

fn filter_targets(opts: &TestFuzz, pat: &str, targets: &[String]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| (!opts.exact && target.contains(pat)) || target.as_str() == pat)
        .cloned()
        .collect()
}

fn executable_target(
    opts: &TestFuzz,
    executable_targets: &[(Executable, Vec<String>)],
) -> Result<(Executable, String)> {
    let mut executable_targets = executable_targets.to_vec();

    ensure!(
        executable_targets.len() <= 1,
        "Found multiple executables with fuzz targets{}: {:#?}",
        match_message(opts),
        executable_targets
    );

    let Some(mut executable_targets) = executable_targets.pop() else {
        bail!("Found no fuzz targets{}", match_message(opts));
    };

    ensure!(
        executable_targets.1.len() <= 1,
        "Found multiple fuzz targets{} in {:?}: {:#?}",
        match_message(opts),
        executable_targets.0,
        executable_targets.1
    );

    #[allow(clippy::expect_used)]
    Ok((
        executable_targets.0,
        executable_targets
            .1
            .pop()
            .expect("Executable with no fuzz targets"),
    ))
}

fn match_message(opts: &TestFuzz) -> String {
    opts.ztarget.as_ref().map_or(String::new(), |pat| {
        format!(
            " {} `{}`",
            if opts.exact { "equal to" } else { "containing" },
            pat
        )
    })
}

fn check_test_fuzz_and_afl_versions(
    executable_targets: &[(Executable, Vec<String>)],
) -> Result<()> {
    let cargo_test_fuzz_version = Version::parse(crate_version!())?;
    for (executable, _) in executable_targets {
        check_dependency_version(
            &executable.name,
            "test-fuzz",
            executable.test_fuzz_version.as_ref(),
            "cargo-test-fuzz",
            &cargo_test_fuzz_version,
        )?;
        check_dependency_version(
            &executable.name,
            "afl",
            executable.afl_version.as_ref(),
            "cargo-afl",
            cached_cargo_afl_version(),
        )?;
    }
    Ok(())
}

fn cached_cargo_afl_version() -> &'static Version {
    #[allow(clippy::unwrap_used)]
    CARGO_AFL_VERSION.get_or_init(|| cargo_afl_version().unwrap())
}

static CARGO_AFL_VERSION: OnceLock<Version> = OnceLock::new();

fn cargo_afl_version() -> Result<Version> {
    let mut command = Command::new("cargo");
    command.args(["afl", "--version"]);
    let output = command
        .output()
        .with_context(|| format!("Could not get output of `{command:?}`"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .strip_prefix("cargo-afl ")
        .and_then(|s| s.split_ascii_whitespace().next())
        .ok_or_else(|| {
            anyhow!(
                "Could not determine `cargo-afl` version. Is it installed? Try `cargo install \
                 cargo-afl`."
            )
        })?;
    Version::parse(version).map_err(Into::into)
}

fn check_dependency_version(
    name: &str,
    dependency: &str,
    dependency_version: Option<&Version>,
    binary: &str,
    binary_version: &Version,
) -> Result<()> {
    if let Some(dependency_version) = dependency_version {
        ensure!(
            as_version_req(dependency_version).matches(binary_version)
                || as_version_req(binary_version).matches(dependency_version),
            "`{}` depends on `{} {}`, which is incompatible with `{} {}`.",
            name,
            dependency,
            dependency_version,
            binary,
            binary_version
        );
        if !as_version_req(dependency_version).matches(binary_version) {
            eprintln!(
                "`{name}` depends on `{dependency} {dependency_version}`, which is newer than \
                 `{binary} {binary_version}`. Consider upgrading with `cargo install {binary} \
                 --force --version '>={dependency_version}'`."
            );
        }
    } else {
        bail!("`{}` does not depend on `{}`", name, dependency)
    }
    Ok(())
}

fn as_version_req(version: &Version) -> VersionReq {
    #[allow(clippy::expect_used)]
    VersionReq::parse(&version.to_string()).expect("Could not parse version as version request")
}

fn consolidate(opts: &TestFuzz, executable_targets: &[(Executable, Vec<String>)]) -> Result<()> {
    assert!(opts.consolidate_all || executable_targets.len() == 1);

    for (executable, targets) in executable_targets {
        assert!(opts.consolidate_all || targets.len() == 1);

        for target in targets {
            let corpus_dir = corpus_directory_from_target(&executable.name, target);
            let crashes_dir = crashes_directory_from_target(&executable.name, target);
            let hangs_dir = hangs_directory_from_target(&executable.name, target);
            let queue_dir = queue_directory_from_target(&executable.name, target);

            for dir in &[crashes_dir, hangs_dir, queue_dir] {
                for entry in read_dir(dir)
                    .with_context(|| format!("`read_dir` failed for `{}`", dir.to_string_lossy()))?
                {
                    let entry = entry.with_context(|| {
                        format!("`read_dir` failed for `{}`", dir.to_string_lossy())
                    })?;
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .map(OsStr::to_string_lossy)
                        .unwrap_or_default();

                    if file_name == "README.txt" || file_name == ".state" {
                        continue;
                    }

                    let data = read(&path).with_context(|| {
                        format!("`read` failed for `{}`", path.to_string_lossy())
                    })?;
                    test_fuzz::runtime::write_data(&corpus_dir, &data).with_context(|| {
                        format!(
                            "`test_fuzz::runtime::write_data` failed for `{}`",
                            corpus_dir.to_string_lossy()
                        )
                    })?;
                }
            }
        }
    }

    Ok(())
}

fn reset(opts: &TestFuzz, executable_targets: &[(Executable, Vec<String>)]) -> Result<()> {
    assert!(opts.reset_all || executable_targets.len() == 1);

    for (executable, targets) in executable_targets {
        assert!(opts.reset_all || targets.len() == 1);

        for target in targets {
            let output_dir = output_directory_from_target(&executable.name, target);
            if !output_dir.exists() {
                continue;
            }
            remove_dir_all(&output_dir).with_context(|| {
                format!(
                    "`remove_dir_all` failed for `{}`",
                    output_dir.to_string_lossy()
                )
            })?;
        }
    }

    Ok(())
}

#[allow(clippy::panic)]
fn flags_and_dir(object: Object, krate: &str, target: &str) -> (Flags, PathBuf) {
    match object {
        Object::Corpus | Object::CorpusInstrumented => (
            Flags::REQUIRES_CARGO_TEST,
            corpus_directory_from_target(krate, target),
        ),
        Object::Crashes | Object::CrashesInstrumented => {
            (Flags::empty(), crashes_directory_from_target(krate, target))
        }
        Object::Hangs | Object::HangsInstrumented => {
            (Flags::empty(), hangs_directory_from_target(krate, target))
        }
        Object::Queue | Object::QueueInstrumented => {
            (Flags::empty(), queue_directory_from_target(krate, target))
        }
        Object::ImplGenericArgs => (
            Flags::REQUIRES_CARGO_TEST | Flags::RAW,
            impl_generic_args_directory_from_target(krate, target),
        ),
        Object::GenericArgs => (
            Flags::REQUIRES_CARGO_TEST | Flags::RAW,
            generic_args_directory_from_target(krate, target),
        ),
    }
}

#[allow(clippy::too_many_lines)]
fn for_each_entry(
    opts: &TestFuzz,
    executable: &Executable,
    target: &str,
    display: bool,
    replay: bool,
    flags: Flags,
    dir: &Path,
) -> Result<()> {
    ensure!(
        dir.exists(),
        "Could not find `{}`{}",
        dir.to_string_lossy(),
        if flags.contains(Flags::REQUIRES_CARGO_TEST) {
            ". Did you remember to run `cargo test`?"
        } else {
            ""
        }
    );

    let mut envs = BASE_ENVS.to_vec();
    envs.push(("AFL_QUIET", "1"));
    if display {
        envs.push(("TEST_FUZZ_DISPLAY", "1"));
    }
    if replay {
        envs.push(("TEST_FUZZ_REPLAY", "1"));
    }
    if opts.backtrace {
        envs.push(("RUST_BACKTRACE", "1"));
    }
    if opts.pretty {
        envs.push(("TEST_FUZZ_PRETTY_PRINT", "1"));
    }

    let args: Vec<String> = vec![
        "--exact",
        &(target.to_owned() + ENTRY_SUFFIX),
        "--nocapture",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut nonempty = false;
    let mut failure = false;
    let mut timeout = false;
    let mut output = false;

    for entry in read_dir(dir)
        .with_context(|| format!("`read_dir` failed for `{}`", dir.to_string_lossy()))?
    {
        let entry =
            entry.with_context(|| format!("`read_dir` failed for `{}`", dir.to_string_lossy()))?;
        let path = entry.path();
        let mut file = File::open(&path)
            .with_context(|| format!("`open` failed for `{}`", path.to_string_lossy()))?;
        let file_name = path
            .file_name()
            .map(OsStr::to_string_lossy)
            .unwrap_or_default();

        if file_name == "README.txt" || file_name == ".state" {
            continue;
        }

        let (buffer, status) = if flags.contains(Flags::RAW) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).with_context(|| {
                format!("`read_to_end` failed for `{}`", path.to_string_lossy())
            })?;
            (buffer, Some(ExitStatus::Exited(0)))
        } else {
            let exec = Exec::cmd(&executable.path)
                .env_extend(&envs)
                .args(&args)
                .stdin(file)
                .stdout(NullFile)
                .stderr(Redirection::Pipe);
            debug!("{exec:?}");
            let mut popen = exec
                .clone()
                .popen()
                .with_context(|| format!("`popen` failed for `{exec:?}`"))?;
            let secs = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
            let time = Duration::from_secs(secs);
            let mut communicator = popen.communicate_start(None).limit_time(time);
            match communicator.read() {
                Ok((_, buffer)) => {
                    let status = popen.wait()?;
                    (buffer.unwrap_or_default(), Some(status))
                }
                Err(CommunicateError {
                    error,
                    capture: (_, buffer),
                }) => {
                    popen
                        .kill()
                        .with_context(|| format!("`kill` failed for `{popen:?}`"))?;
                    if error.kind() != std::io::ErrorKind::TimedOut {
                        return Err(anyhow!(error));
                    }
                    let _ = popen.wait()?;
                    (buffer.unwrap_or_default(), None)
                }
            }
        };

        print!("{file_name}: ");
        if let Some(last) = buffer.last() {
            print!("{}", String::from_utf8_lossy(&buffer));
            if last != &b'\n' {
                println!();
            }
            output = true;
        }
        status.map_or_else(
            || {
                println!("Timeout");
                timeout = true;
            },
            |status| {
                if !flags.contains(Flags::RAW) && buffer.is_empty() {
                    println!("{status:?}");
                }
                failure |= !status.success();
            },
        );

        nonempty = true;
    }

    assert!(!(!nonempty && (failure || timeout || output)));

    if !nonempty {
        eprintln!(
            "Nothing to {}.",
            match (display, replay) {
                (true, true) => "display/replay",
                (true, false) => "display",
                (false, true) => "replay",
                (false, false) => unreachable!(),
            }
        );
        return Ok(());
    }

    if !failure && !timeout && !output {
        eprintln!("No output on stderr detected.");
        return Ok(());
    }

    if (failure || timeout) && !replay {
        eprintln!(
            "Encountered a {} while not replaying. A buggy Debug implementation perhaps?",
            if failure {
                "failure"
            } else if timeout {
                "timeout"
            } else {
                unreachable!()
            }
        );
        return Ok(());
    }

    Ok(())
}

fn flatten_executable_targets(
    opts: &TestFuzz,
    executable_targets: Vec<(Executable, Vec<String>)>,
) -> Result<Vec<(Executable, String)>> {
    let executable_targets = executable_targets
        .into_iter()
        .flat_map(|(executable, targets)| {
            targets
                .into_iter()
                .map(move |target| (executable.clone(), target))
        })
        .collect::<Vec<_>>();

    ensure!(
        !executable_targets.is_empty(),
        "Found no fuzz targets{}",
        match_message(opts)
    );

    Ok(executable_targets)
}

struct Config {
    ui: bool,
    sufficient_cpus: bool,
    first_run: bool,
}

struct Child {
    exec: String,
    popen: StdChild,
    receiver: Receiver,
    unprinted_data: Vec<u8>,
    output_buffer: VecDeque<String>,
    time_limit_was_reached: bool,
    testing_aborted_programmatically: bool,
}

impl Child {
    fn read_lines(&mut self) -> Result<String> {
        loop {
            let mut buf = [0; 4096];
            let n = match self.receiver.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error.into()),
            };
            self.unprinted_data.extend_from_slice(&buf[0..n]);
        }
        if let Some(i) = self.unprinted_data.iter().rev().position(|&c| c == b'\n') {
            let mut buf = self.unprinted_data.split_off(self.unprinted_data.len() - i);
            std::mem::swap(&mut self.unprinted_data, &mut buf);
            String::from_utf8(buf).map_err(Into::into)
        } else {
            Ok(String::new())
        }
    }

    fn print_line(&mut self, opts: &TestFuzz, line: String) {
        if opts.no_ui {
            println!("{line}");
        } else {
            self.output_buffer.push_back(line);
        }
    }

    fn refresh(opts: &TestFuzz, n_children: usize, children: &mut [Option<Child>]) {
        if opts.no_ui {
            return;
        }

        cursor_to_home_position();

        let termsize::Size { rows, cols } = termsize::get().unwrap();
        let rows = rows as usize;
        let cols = cols as usize;

        // smoelius: `n_children` - 1 lines for dividers plus one line at the bottom of the
        // terminal to hold the cursor.
        let Some(n_available_rows) = rows.checked_sub(n_children) else {
            return;
        };

        let children = children.iter_mut().flatten().collect::<Vec<_>>();

        assert_eq!(n_children, children.len());

        for (i_child, child) in children.into_iter().enumerate() {
            if i_child != 0 {
                println!("{:-<cols$}", "");
            }
            let n_child_rows = n_available_rows / n_children
                + if i_child < n_available_rows % n_children {
                    1
                } else {
                    0
                };
            let n_lines_to_skip = child.output_buffer.len().saturating_sub(n_child_rows);
            child.output_buffer.drain(..n_lines_to_skip);
            for i in 0..n_child_rows {
                if let Some(line) = child.output_buffer.get(i) {
                    let prefix = prefix_with_width(line, cols);
                    print!("{prefix}");
                }
                clear_to_end_of_line();
                println!();
            }
        }
    }
}

fn cursor_to_home_position() {
    print!("\x1b[H");
}

fn clear_to_end_of_line() {
    print!("\x1b[0K");
}

#[cfg_attr(dylint_lib = "supplementary", allow(commented_out_code))]
fn prefix_with_width(s: &str, width: usize) -> &str {
    let mut min = 0;
    let mut max = s.len();
    while min < max {
        let mid = (min + max) / 2;
        let prefix = strip_ansi_escapes::strip(&s[..mid]);
        if prefix.len() < width {
            min = mid + 1;
        } else {
            // width <= prefix.len()
            max = mid;
        }
    }
    assert_eq!(min, max);
    &s[..min]
}

#[allow(clippy::too_many_lines)]
fn fuzz(opts: &TestFuzz, executable_targets: &[(Executable, String)]) -> Result<()> {
    auto_generate_corpora(executable_targets)?;

    let mut config = Config {
        ui: !opts.no_ui,
        sufficient_cpus: true,
        first_run: true,
    };

    if let (false, [(executable, target)]) = (opts.exit_code, executable_targets) {
        let mut command = fuzz_command(opts, &config, executable, target);
        let status = command
            .status()
            .with_context(|| format!("Could not get status of `{command:?}`"))?;
        ensure!(status.success(), "Command failed: {:?}", command);
        return Ok(());
    }

    let n_cpus = std::cmp::min(
        opts.cpus.unwrap_or_else(|| num_cpus::get() - 1),
        num_cpus::get(),
    );

    ensure!(n_cpus >= 1, "Number of cpus must be greater than zero");

    config.sufficient_cpus = n_cpus >= executable_targets.len();

    if !config.sufficient_cpus {
        ensure!(
            opts.max_total_time.is_none(),
            "--max-total-time cannot be used when number of cpus ({n_cpus}) is less than number \
             of fuzz targets ({})",
            executable_targets.len()
        );

        eprintln!(
            "Number of cpus ({n_cpus}) is less than number of fuzz targets ({}); fuzzing each for \
             {} seconds",
            executable_targets.len(),
            opts.slice
        );
    }

    let mut n_children = 0;
    let mut i_task = 0;
    let mut executable_targets_iter = executable_targets.iter().cycle();
    let mut poll = Poll::new().with_context(|| "`Poll::new` failed")?;
    let mut events = Events::with_capacity(128);
    let mut children = vec![(); executable_targets.len()]
        .into_iter()
        .map(|()| None::<Child>)
        .collect::<Vec<_>>();
    let mut i_target_prev = executable_targets.len();

    // Track failed targets to detect when all targets fail
    let mut failed_targets = std::collections::HashSet::new();

    loop {
        Child::refresh(opts, n_children, children.as_mut_slice());

        // If all targets have failed, terminate gracefully
        if failed_targets.len() == executable_targets.len() {
            bail!("All targets failed to start");
        }

        if n_children < n_cpus && (i_task < executable_targets.len() || !config.sufficient_cpus) {
            let Some((executable, target)) = executable_targets_iter.next() else {
                unreachable!();
            };

            let i_target = i_task % executable_targets.len();

            // Skip targets that have already failed
            if failed_targets.contains(&i_target) {
                i_task += 1;
                continue;
            }

            // smoelius: Here is how I think this condition could arise. Suppose there are three
            // targets and two cpus, and that tasks 0 and 1 are currently running. Suppose then that
            // task 1 completes and task 2 cannot be started for some reason, so cargo-test-fuzz
            // tries to start task 3. Note that tasks 0 and 3 correspond to the same target. So if
            // task 0 is still running, `children[i_target]` will be `Some(..)`.
            if children[i_target].is_some() {
                assert!(!config.sufficient_cpus);
                i_task += 1;
                continue;
            }

            config.first_run = i_task < executable_targets.len();

            // smoelius: If this is not the target's first run, then there must be insufficient
            // cpus.
            assert!(config.first_run || !config.sufficient_cpus);

            let mut command = fuzz_command(opts, &config, executable, target);

            let exec = format!("{command:?}");
            command.stdout(Stdio::piped());
            let mut popen = command
                .spawn()
                .with_context(|| format!("Could not spawn `{exec:?}`"))?;
            let stdout = popen
                .stdout
                .take()
                .ok_or_else(|| anyhow!("Could not get output of `{exec:?}`"))?;
            let mut receiver = Receiver::from(stdout);
            receiver
                .set_nonblocking(true)
                .with_context(|| "Could not make receiver non-blocking")?;
            poll.registry()
                .register(&mut receiver, Token(i_target), Interest::READABLE)
                .with_context(|| "Could not register receiver")?;
            children[i_target] = Some(Child {
                exec,
                popen,
                receiver,
                unprinted_data: Vec::new(),
                output_buffer: VecDeque::new(),
                time_limit_was_reached: false,
                testing_aborted_programmatically: false,
            });

            n_children += 1;
            i_task += 1;
            continue;
        }

        if n_children == 0 {
            assert!(config.sufficient_cpus);
            assert!(i_task >= executable_targets.len());
            break;
        }

        poll.poll(&mut events, None)
            .with_context(|| "`poll` failed")?;

        for event in &events {
            let Token(i_target) = event.token();
            let (_, target) = &executable_targets[i_target];
            #[allow(clippy::panic)]
            let child = children[i_target]
                .as_mut()
                .unwrap_or_else(|| panic!("Child for token {i_target} should exist"));

            let s = child.read_lines()?;
            for line in s.lines() {
                if line.contains("Time limit was reached") {
                    child.time_limit_was_reached = true;
                }
                if line.contains("+++ Testing aborted programmatically +++") {
                    child.testing_aborted_programmatically = true;
                }
                if opts.no_ui
                    && i_target_prev < executable_targets.len()
                    && i_target_prev != i_target
                {
                    println!("---");
                }
                child.print_line(opts, format!("{target}: {line}"));
                i_target_prev = i_target;
            }

            if event.is_read_closed() {
                #[allow(clippy::panic)]
                let mut child = children[i_target]
                    .take()
                    .unwrap_or_else(|| panic!("Child for token {i_target} should exist"));
                poll.registry()
                    .deregister(&mut child.receiver)
                    .with_context(|| "Could not deregister receiver")?;
                n_children -= 1;

                let status = child
                    .popen
                    .wait()
                    .with_context(|| format!("`wait` failed for `{:?}`", child.popen))?;

                if !status.success() {
                    eprintln!(
                        "Warning: Command failed for target {}: {:?}\nstdout: ```\n{}\n```",
                        target,
                        child.exec,
                        itertools::join(child.output_buffer.iter(), "\n")
                    );
                    failed_targets.insert(i_target);
                    continue;
                }

                if !child.testing_aborted_programmatically {
                    eprintln!(
                        r#"Warning: Could not find "Testing aborted programmatically" in command output for target {}: {:?}"#,
                        target, child.exec
                    );
                    failed_targets.insert(i_target);
                    continue;
                }

                if opts.exit_code && !child.time_limit_was_reached {
                    exit(1);
                }
            }
        }
    }

    Ok(())
}

fn fuzz_command(
    opts: &TestFuzz,
    config: &Config,
    executable: &Executable,
    target: &str,
) -> Command {
    let input_dir = if opts.resume || !config.first_run {
        "-".to_owned()
    } else {
        corpus_directory_from_target(&executable.name, target)
            .to_string_lossy()
            .into_owned()
    };

    let output_dir = output_directory_from_target(&executable.name, target);
    create_dir_all(&output_dir).unwrap_or_default();

    let mut envs = BASE_ENVS.to_vec();
    if !config.ui {
        envs.push(("AFL_NO_UI", "1"));
    }
    if opts.run_until_crash {
        envs.push(("AFL_BENCH_UNTIL_CRASH", "1"));
    }

    // smoelius: Passing `-c-` disables the cmplog fork server. Use of cmplog with a target that
    // spawns subprocesses (like libtest does) can leave orphan processes running. This can cause
    // problems, e.g., if those processes hold flocks. See:
    // https://github.com/AFLplusplus/AFLplusplus/issues/2178
    let mut args = vec![];
    args.extend(
        vec![
            "afl",
            "fuzz",
            "-c-",
            "-i",
            &input_dir,
            "-o",
            &output_dir.to_string_lossy(),
            "-M",
            "default",
        ]
        .into_iter()
        .map(String::from),
    );
    if !config.sufficient_cpus {
        args.extend(["-V".to_owned(), opts.slice.to_string()]);
    } else if let Some(max_total_time) = opts.max_total_time {
        args.extend(["-V".to_owned(), max_total_time.to_string()]);
    }
    if let Some(timeout) = opts.timeout {
        args.extend(["-t".to_owned(), format!("{}", timeout * MILLIS_PER_SEC)]);
    }
    args.extend(opts.zzargs.clone());
    args.extend(
        vec![
            "--",
            &executable.path.to_string_lossy(),
            "--exact",
            &(target.to_owned() + ENTRY_SUFFIX),
        ]
        .into_iter()
        .map(String::from),
    );

    let mut command = Command::new("cargo");
    command.envs(envs).args(args);
    debug!("{command:?}");
    command
}

fn auto_generate_corpora(executable_targets: &[(Executable, String)]) -> Result<()> {
    for (executable, target) in executable_targets {
        let corpus_dir = corpus_directory_from_target(&executable.name, target);
        if !corpus_dir.exists() {
            eprintln!(
                "Could not find `{}`. Trying to auto-generate it...",
                corpus_dir.to_string_lossy(),
            );
            auto_generate_corpus(executable, target)?;
            ensure!(
                corpus_dir.exists(),
                "Could not find or auto-generate `{}`. Please ensure `{}` is tested.",
                corpus_dir.to_string_lossy(),
                target
            );
            eprintln!("Auto-generated `{}`.", corpus_dir.to_string_lossy());
        }
    }

    Ok(())
}

fn auto_generate_corpus(executable: &Executable, target: &str) -> Result<()> {
    let mut command = Command::new(&executable.path);
    command.args(["--exact", &(target.to_owned() + AUTO_GENERATED_SUFFIX)]);
    debug!("{command:?}");
    let status = command
        .status()
        .with_context(|| format!("Could not get status of `{command:?}`"))?;

    ensure!(status.success(), "Command failed: {:?}", command);

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::{build, executable_targets, TestFuzz};
    use predicates::prelude::*;
    use std::{env::set_current_dir, process::Command};
    use testing::CommandExt;

    // smoelius: The only other tests in this file are the ones generated by the `test_fuzz` macro
    // for `filter_executable_targets`, and they are not affected by the change in directory.
    #[cfg_attr(dylint_lib = "supplementary", allow(non_thread_safe_call_in_test))]
    #[test]
    fn warn_if_test_fuzz_not_enabled() {
        const WARNING: &str = "If you are trying to run a test-fuzz-generated fuzzing harness, be \
                               sure to run with `TEST_FUZZ=1`.";

        set_current_dir("../examples").unwrap();

        let executables = build(&TestFuzz::default(), false, false).unwrap();

        let executable_targets = executable_targets(&executables).unwrap();

        // smoelius: Any example executable should work _except_ the ones that use
        // `only_generic_args`. Currently, those are `generic` and `unserde`.
        let (executable, _) = executable_targets
            .iter()
            .filter(|(executable, _)| !["generic", "unserde"].contains(&executable.name.as_str()))
            .next()
            .unwrap();

        for enable in [false, true] {
            let mut command = Command::new(&executable.path);
            if enable {
                command.env("TEST_FUZZ", "1");
            }
            let assert = command.logged_assert().success();
            if enable {
                assert.stderr(predicate::str::contains(WARNING).not());
            } else {
                assert.stderr(predicate::str::contains(WARNING));
            }
        }
    }
}
