#![cfg_attr(dylint_lib = "crate_wide_allow", allow(crate_wide_allow))]
#![allow(clippy::use_self)]
#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::panic)]

use anyhow::{anyhow, bail, ensure, Context, Result};
use bitflags::bitflags;
use cargo_metadata::{
    Artifact, ArtifactProfile, CargoOpt, Message, Metadata, MetadataCommand, Package, PackageId,
};
use clap::{crate_version, ArgEnum};
use heck::ToKebabCase;
use internal::dirs::{
    concretizations_directory_from_target, corpus_directory_from_target,
    crashes_directory_from_target, hangs_directory_from_target,
    impl_concretizations_directory_from_target, output_directory_from_target,
    queue_directory_from_target, target_directory,
};
use lazy_static::lazy_static;
use log::debug;
use semver::Version;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::{Debug, Formatter},
    fs::{create_dir_all, read, read_dir, remove_dir_all, File},
    io::{BufRead, Read},
    iter,
    path::{Path, PathBuf},
    process::{exit, Command},
    sync::Mutex,
    time::Duration,
};
use strum_macros::Display;
use subprocess::{CommunicateError, Exec, ExitStatus, NullFile, Redirection};

mod transition;
#[allow(deprecated)]
pub use transition::cargo_test_fuzz;

const AUTO_GENERATED_SUFFIX: &str = "_fuzz::auto_generate";
const ENTRY_SUFFIX: &str = "_fuzz::entry";

const BASE_ENVS: &[(&str, &str)] = &[("TEST_FUZZ", "1"), ("TEST_FUZZ_WRITE", "0")];

const DEFAULT_TIMEOUT: u64 = 1000;

const NANOS_PER_MILLI: u64 = 1_000_000;

bitflags! {
    struct Flags: u8 {
        const REQUIRES_CARGO_TEST = 0b0000_0001;
        const RAW = 0b0000_0010;
    }
}

#[derive(ArgEnum, Clone, Copy, Debug, Display, Deserialize, PartialEq, Eq, Serialize)]
#[remain::sorted]
enum Object {
    Concretizations,
    Corpus,
    CorpusInstrumented,
    Crashes,
    Hangs,
    ImplConcretizations,
    Queue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[remain::sorted]
struct TestFuzz {
    backtrace: bool,
    consolidate: bool,
    consolidate_all: bool,
    display: Option<Object>,
    exact: bool,
    exit_code: bool,
    features: Vec<String>,
    list: bool,
    manifest_path: Option<String>,
    no_default_features: bool,
    no_instrumentation: bool,
    no_run: bool,
    no_ui: bool,
    package: Option<String>,
    persistent: bool,
    pretty_print: bool,
    replay: Option<Object>,
    reset: bool,
    reset_all: bool,
    resume: bool,
    run_until_crash: bool,
    test: Option<String>,
    timeout: Option<u64>,
    verbose: bool,
    ztarget: Option<String>,
    zzargs: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Executable {
    path: PathBuf,
    name: String,
    test_fuzz_version: Option<Version>,
    afl_version: Option<Version>,
}

impl Debug for Executable {
    #[cfg_attr(
        dylint_lib = "non_local_effect_before_error_return",
        allow(non_local_effect_before_error_return)
    )]
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

fn run(opts: TestFuzz) -> Result<()> {
    let opts = {
        let mut opts = opts;
        if opts.exit_code {
            opts.no_ui = true;
        }
        if opts.list
            || matches!(
                opts.display,
                Some(Object::Corpus | Object::ImplConcretizations | Object::Concretizations)
            )
            || opts.replay == Some(Object::Corpus)
        {
            opts.no_instrumentation = true;
        }
        opts
    };

    if let Some(object) = opts.replay {
        ensure!(
            !matches!(
                object,
                Object::ImplConcretizations | Object::Concretizations
            ),
            "`--replay {}` is invalid.",
            object.to_string().to_kebab_case()
        );
    }

    cache_cargo_afl_version()?;

    let display = opts.display.is_some();

    let replay = opts.replay.is_some();

    let executables = build(&opts, display || replay)?;

    let mut executable_targets = executable_targets(&executables)?;

    if let Some(pat) = &opts.ztarget {
        executable_targets = filter_executable_targets(&opts, pat, &executable_targets);
    }

    check_test_fuzz_and_afl_versions(&executable_targets)?;

    if opts.list {
        println!("{:#?}", executable_targets);
        return Ok(());
    }

    if opts.no_run {
        return Ok(());
    }

    if opts.consolidate_all || opts.reset_all {
        if opts.consolidate_all {
            consolidate(&opts, &executable_targets)?;
        }
        return reset(&opts, &executable_targets);
    }

    let (executable, target) = executable_target(&opts, &executable_targets)?;

    if opts.consolidate || opts.reset {
        if opts.consolidate {
            consolidate(&opts, &executable_targets)?;
        }
        return reset(&opts, &executable_targets);
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
        .unwrap_or((Flags::empty(), PathBuf::default()));

    if display || replay {
        return for_each_entry(&opts, &executable, &target, display, replay, flags, &dir);
    }

    if opts.no_instrumentation {
        eprintln!("Stopping before fuzzing since --no-instrumentation was specified.");
        return Ok(());
    }

    fuzz(&opts, &executable, &target).map_err(|error| {
        if opts.exit_code {
            eprintln!("{:?}", error);
            exit(2);
        }
        error
    })
}

fn build(opts: &TestFuzz, quiet: bool) -> Result<Vec<Executable>> {
    let metadata = metadata(opts)?;

    let mut args = vec![];
    if !opts.no_instrumentation {
        args.extend_from_slice(&["afl"]);
    }
    args.extend_from_slice(&["test", "--frozen", "--offline", "--no-run"]);
    if opts.no_default_features {
        args.extend_from_slice(&["--no-default-features"]);
    }
    for features in &opts.features {
        args.extend_from_slice(&["--features", features]);
    }
    let target_dir = target_directory(true);
    let target_dir_str = target_dir.to_string_lossy();
    if !opts.no_instrumentation {
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
    // `AFL_QUIET=1` doesn't work here, so pipe standard error to /dev/null.
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
    if quiet && !opts.verbose {
        exec = exec.stderr(NullFile);
    }
    debug!("{:?}", exec);
    let mut popen = exec.clone().popen()?;
    let artifacts = popen
        .stdout
        .take()
        .map_or(Ok(vec![]), |stream| -> Result<_> {
            let reader = std::io::BufReader::new(stream);
            let artifacts: Vec<Artifact> = Message::parse_stream(reader)
                .filter_map(|result| match result {
                    Ok(message) => match message {
                        Message::CompilerArtifact(artifact) => Some(Ok(artifact)),
                        _ => None,
                    },
                    Err(err) => Some(Err(err)),
                })
                .collect::<std::result::Result<_, std::io::Error>>()
                .with_context(|| format!("`parse_stream` failed for `{:?}`", exec))?;
            Ok(artifacts)
        })?;
    let status = popen
        .wait()
        .with_context(|| format!("`wait` failed for `{:?}`", popen))?;

    // smoelius: If the command failed, re-execute it without --message-format=json. This is easier
    // than trying to capture and colorize `CompilerMessage`s like Cargo does.
    if !status.success() {
        let mut popen = Exec::cmd("cargo").args(&args).popen()?;
        let status = popen
            .wait()
            .with_context(|| format!("`wait` failed for `{:?}`", popen))?;
        ensure!(
            !status.success(),
            "Command succeeded unexpectedly: {:?}",
            exec,
        );
        bail!("Command failed: {:?}", exec);
    }

    let executables = artifacts
        .into_iter()
        .map(|artifact| {
            if let Artifact {
                package_id,
                target: build_target,
                profile: ArtifactProfile { test: true, .. },
                executable: Some(executable),
                ..
            } = artifact
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
        .filter(|executable_targets| !executable_targets.1.is_empty())
        .collect())
}

fn targets(executable: &Path) -> Result<Vec<String>> {
    let exec = Exec::cmd(executable)
        .env_extend(&[("AFL_QUIET", "1")])
        .args(&["--list"])
        .stderr(NullFile);
    debug!("{:?}", exec);
    let stream = exec.clone().stream_stdout()?;

    // smoelius: A test executable's --list output ends with an empty line followed by
    // "M tests, N benchmarks." Stop at the empty line.
    let mut targets = Vec::<String>::default();
    for line in std::io::BufReader::new(stream).lines() {
        let line = line.with_context(|| format!("Could not get output of `{:?}`", exec))?;
        if line.is_empty() {
            break;
        }
        let line = if let Some(line) = line.strip_suffix(": test") {
            line
        } else {
            continue;
        };
        let line = if let Some(line) = line.strip_suffix(ENTRY_SUFFIX) {
            line
        } else {
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
            if !targets.is_empty() {
                Some((executable.clone(), targets))
            } else {
                None
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

#[allow(clippy::expect_used)]
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

    let mut executable_targets = if let Some(executable_targets) = executable_targets.pop() {
        executable_targets
    } else {
        bail!("Found no fuzz targets{}", match_message(opts));
    };

    ensure!(
        executable_targets.1.len() <= 1,
        "Found multiple fuzz targets{} in {:?}: {:#?}",
        match_message(opts),
        executable_targets.0,
        executable_targets.1
    );

    Ok((
        executable_targets.0,
        executable_targets
            .1
            .pop()
            .expect("Executable with no fuzz targets"),
    ))
}

fn match_message(opts: &TestFuzz) -> String {
    opts.ztarget.as_ref().map_or("".to_owned(), |pat| {
        format!(
            " {} `{}`",
            if opts.exact { "equal to" } else { "containing" },
            pat
        )
    })
}

#[allow(clippy::expect_used)]
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
            "cargo-test-fuzz",
        )?;
        check_dependency_version(
            &executable.name,
            "afl",
            executable.afl_version.as_ref(),
            "cargo-afl",
            CARGO_AFL_VERSION
                .lock()
                .expect("Could not lock `CARGO_AFL_VERSION`")
                .as_ref()
                .expect("Could not determine `cargo-afl` version"),
            "afl",
        )?;
    }
    Ok(())
}

fn cache_cargo_afl_version() -> Result<()> {
    let cargo_afl_version = cargo_afl_version()?;
    let mut lock = CARGO_AFL_VERSION
        .lock()
        .map_err(|error| anyhow!(error.to_string()))?;
    *lock = Some(cargo_afl_version);
    Ok(())
}

lazy_static! {
    static ref CARGO_AFL_VERSION: Mutex<Option<Version>> = Mutex::new(None);
}

fn cargo_afl_version() -> Result<Version> {
    let mut command = Command::new("cargo");
    command.args(&["afl", "--version"]);
    let output = command
        .output()
        .with_context(|| format!("Could not get output of `{:?}`", command))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.strip_prefix("cargo-afl ").ok_or_else(|| {
        anyhow!(
            "Could not determine `cargo-afl` version. Is it installed? Try `cargo install afl`."
        )
    })?;
    Version::parse(version.trim_end()).map_err(Into::into)
}

fn check_dependency_version(
    name: &str,
    dependency: &str,
    dependency_version: Option<&Version>,
    binary: &str,
    binary_version: &Version,
    krate: &str,
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
                "`{}` depends on `{} {}`, which is newer than `{} {}`. Consider upgrading with \
                `cargo install {} --force --version '>={}'`.",
                name,
                dependency,
                dependency_version,
                binary,
                binary_version,
                krate,
                dependency_version
            );
        }
    } else {
        bail!("`{}` does not depend on `{}`", name, dependency)
    }
    Ok(())
}

#[allow(clippy::expect_used)]
fn as_version_req(version: &Version) -> VersionReq {
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

fn flags_and_dir(object: Object, krate: &str, target: &str) -> (Flags, PathBuf) {
    match object {
        Object::Corpus | Object::CorpusInstrumented => (
            Flags::REQUIRES_CARGO_TEST,
            corpus_directory_from_target(krate, target),
        ),
        Object::Crashes => (Flags::empty(), crashes_directory_from_target(krate, target)),
        Object::Hangs => (Flags::empty(), hangs_directory_from_target(krate, target)),
        Object::Queue => (Flags::empty(), queue_directory_from_target(krate, target)),
        Object::ImplConcretizations => (
            Flags::REQUIRES_CARGO_TEST | Flags::RAW,
            impl_concretizations_directory_from_target(krate, target),
        ),
        Object::Concretizations => (
            Flags::REQUIRES_CARGO_TEST | Flags::RAW,
            concretizations_directory_from_target(krate, target),
        ),
    }
}

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
    if opts.pretty_print {
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
            debug!("{:?}", exec);
            let mut popen = exec
                .clone()
                .popen()
                .with_context(|| format!("`popen` failed for `{:?}`", exec))?;
            let millis = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
            let time = Duration::from_millis(millis);
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
                        .with_context(|| format!("`kill` failed for `{:?}`", popen))?;
                    if error.kind() != std::io::ErrorKind::TimedOut {
                        return Err(anyhow!(error));
                    }
                    let _ = popen.wait()?;
                    (buffer.unwrap_or_default(), None)
                }
            }
        };

        print!("{}: ", file_name);
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
                    println!("{:?}", status);
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

fn fuzz(opts: &TestFuzz, executable: &Executable, target: &str) -> Result<()> {
    let input_dir = if opts.resume {
        "-".to_owned()
    } else {
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
        corpus_dir.to_string_lossy().into_owned()
    };

    let output_dir = output_directory_from_target(&executable.name, target);
    create_dir_all(&output_dir).unwrap_or_default();

    let mut envs = BASE_ENVS.to_vec();
    if opts.no_ui {
        envs.push(("AFL_NO_UI", "1"));
    }
    if opts.run_until_crash {
        envs.push(("AFL_BENCH_UNTIL_CRASH", "1"));
    }

    let mut args = vec![];
    args.extend(
        vec![
            "afl",
            "fuzz",
            "-i",
            &input_dir,
            "-o",
            &output_dir.to_string_lossy(),
            "-D",
            "-M",
            "default",
        ]
        .into_iter()
        .map(String::from),
    );
    if let Some(timeout) = opts.timeout {
        args.extend(vec![
            "-t".to_owned(),
            format!("{}", timeout * NANOS_PER_MILLI),
        ]);
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

    if !opts.exit_code {
        let mut command = Command::new("cargo");
        command.envs(envs).args(args);
        debug!("{:?}", command);
        let status = command
            .status()
            .with_context(|| format!("Could not get status of `{:?}`", command))?;

        ensure!(status.success(), "Command failed: {:?}", command);
    } else {
        let exec = Exec::cmd("cargo")
            .env_extend(&envs)
            .args(&args)
            .stdout(Redirection::Pipe);
        debug!("{:?}", exec);
        let mut popen = exec.clone().popen()?;
        let stdout = popen
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Could not get output of `{:?}`", exec))?;
        let mut time_limit_was_reached = false;
        let mut testing_aborted_programmatically = false;
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.with_context(|| format!("Could not get output of `{:?}`", exec))?;
            if line.contains("Time limit was reached") {
                time_limit_was_reached = true;
            }
            if line.contains("+++ Testing aborted programmatically +++") {
                testing_aborted_programmatically = true;
            }
            println!("{}", line);
        }
        let status = popen
            .wait()
            .with_context(|| format!("`wait` failed for `{:?}`", popen))?;

        if !testing_aborted_programmatically || !status.success() {
            bail!("Command failed: {:?}", exec);
        }

        if !time_limit_was_reached {
            exit(1);
        }
    }

    Ok(())
}

fn auto_generate_corpus(executable: &Executable, target: &str) -> Result<()> {
    let mut command = Command::new(&executable.path);
    command.args(&["--exact", &(target.to_owned() + AUTO_GENERATED_SUFFIX)]);
    debug!("{:?}", command);
    let status = command
        .status()
        .with_context(|| format!("Could not get status of `{:?}`", command))?;

    ensure!(status.success(), "Command failed: {:?}", command);

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    #![allow(clippy::unwrap_used)]

    use super::cargo_test_fuzz as cargo;
    use anyhow::Result;

    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    #[test]
    fn build_no_instrumentation_with_target() {
        cargo_test_fuzz(&[
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format().as_feature()),
            "--no-run",
            "--no-instrumentation",
            "target",
        ])
        .unwrap();
    }

    fn cargo_test_fuzz(args: &[&str]) -> Result<()> {
        let mut cargo_args = vec!["cargo-test-fuzz", "test-fuzz"];
        cargo_args.extend_from_slice(args);
        cargo(&cargo_args)
    }
}
