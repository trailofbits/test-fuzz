#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::panic)]

use anyhow::{anyhow, bail, ensure, Result};
use cargo_metadata::{
    Artifact, ArtifactProfile, Message, Metadata, MetadataCommand, Package, PackageId,
};
use clap::{crate_version, Clap};
use dirs::{
    corpus_directory_from_target, crashes_directory_from_target, hangs_directory_from_target,
    output_directory_from_target, queue_directory_from_target, target_directory,
};
use log::debug;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::{Debug, Formatter},
    fs::{create_dir_all, read, read_dir, remove_dir_all, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use subprocess::{CommunicateError, Exec, NullFile, Redirection};

const ENTRY_SUFFIX: &str = "_fuzz::entry";

const BASE_ENVS: &[(&str, &str)] = &[("TEST_FUZZ", "1"), ("TEST_FUZZ_WRITE", "0")];

const DEFAULT_TIMEOUT: u64 = 1000;

const NANOS_PER_MILLI: u64 = 1_000_000;

#[derive(Clap, Debug)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    TestFuzz(TestFuzz),
}

// smoelius: Wherever possible, try to reuse cargo test and libtest option names.
#[derive(Clap, Clone, Debug, Deserialize, Serialize)]
#[clap(version = crate_version!())]
struct TestFuzz {
    #[clap(long, about = "Display backtraces")]
    backtrace: bool,
    #[clap(
        long,
        about = "Move one target's crashes, hangs, and work queue to its corpus; to consolidate \
        all targets, use --consolidate-all"
    )]
    consolidate: bool,
    #[clap(long, hidden = true)]
    consolidate_all: bool,
    #[clap(
        long,
        about = "Display corpus using uninstrumented fuzz target; to display with instrumentation, \
        use --display-corpus-instrumented"
    )]
    display_corpus: bool,
    #[clap(long, hidden = true)]
    display_corpus_instrumented: bool,
    #[clap(long, about = "Display crashes")]
    display_crashes: bool,
    #[clap(long, about = "Display hangs")]
    display_hangs: bool,
    #[clap(long, about = "Display work queue")]
    display_queue: bool,
    #[clap(long, about = "Target name is an exact name rather than a substring")]
    exact: bool,
    #[clap(long, about = "List fuzz targets")]
    list: bool,
    #[clap(long, about = "Resume target's last fuzzing session")]
    resume: bool,
    #[clap(
        long,
        about = "Compile without instrumentation (for testing build process)"
    )]
    no_instrumentation: bool,
    #[clap(long, about = "Compile, but don't fuzz")]
    no_run: bool,
    #[clap(long, about = "Disable user interface")]
    no_ui: bool,
    #[clap(long, about = "Enable persistent mode fuzzing")]
    persistent: bool,
    #[clap(long, about = "Pretty-print debug output when displaying/replaying")]
    pretty_print: bool,
    #[clap(short, long, about = "Package containing fuzz target")]
    package: Option<String>,
    #[clap(
        long,
        about = "Replay corpus using uninstrumented fuzz target; to replay with instrumentation, \
        use --replay-corpus-instrumented"
    )]
    replay_corpus: bool,
    #[clap(long, hidden = true)]
    replay_corpus_instrumented: bool,
    #[clap(long, about = "Replay crashes")]
    replay_crashes: bool,
    #[clap(long, about = "Replay hangs")]
    replay_hangs: bool,
    #[clap(long, about = "Replay work queue")]
    replay_queue: bool,
    #[clap(
        long,
        about = "Clear fuzzing data for one target, but leave corpus intact; to reset all \
        targets, use --reset-all"
    )]
    reset: bool,
    #[clap(long, hidden = true)]
    reset_all: bool,
    #[clap(long, about = "Stop fuzzing once a crash is found")]
    run_until_crash: bool,
    #[clap(long, about = "String that fuzz target's name must contain")]
    target: Option<String>,
    #[clap(
        long,
        about = "Number of milliseconds to consider a hang when fuzzing or replaying (equivalent \
        to `-- -t <timeout>` when fuzzing)"
    )]
    timeout: Option<u64>,
    #[clap(last = true, about = "Arguments for the fuzzer")]
    args: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Executable {
    path: PathBuf,
    name: String,
    test_fuzz_version: Option<Version>,
    afl_version: Option<Version>,
}

impl Debug for Executable {
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

pub fn cargo_test_fuzz<T: AsRef<OsStr>>(args: &[T]) -> Result<()> {
    let opts = {
        let SubCommand::TestFuzz(mut opts) = Opts::parse_from(args).subcmd;
        if opts.list || opts.display_corpus || opts.replay_corpus {
            opts.no_instrumentation = true;
        }
        opts
    };

    let display = opts.display_corpus
        || opts.display_corpus_instrumented
        || opts.display_crashes
        || opts.display_hangs
        || opts.display_queue;

    let replay = opts.replay_corpus
        || opts.replay_corpus_instrumented
        || opts.replay_crashes
        || opts.replay_hangs
        || opts.replay_queue;

    let executables = build(&opts, display || replay)?;

    let mut executable_targets = executable_targets(&executables)?;

    if let Some(pat) = &opts.target {
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

    let dir = if opts.display_corpus
        || opts.display_corpus_instrumented
        || opts.replay_corpus
        || opts.replay_corpus_instrumented
    {
        corpus_directory_from_target(&executable.name, &target)
    } else if opts.display_crashes || opts.replay_crashes {
        crashes_directory_from_target(&executable.name, &target)
    } else if opts.display_hangs || opts.replay_hangs {
        hangs_directory_from_target(&executable.name, &target)
    } else if opts.display_queue || opts.replay_queue {
        queue_directory_from_target(&executable.name, &target)
    } else {
        PathBuf::default()
    };

    if display || replay {
        return for_each_entry(&opts, &executable, &target, display, replay, &dir);
    }

    if opts.no_instrumentation {
        eprintln!("Stopping before fuzzing since --no-instrumentation was specified.");
        return Ok(());
    }

    fuzz(&opts, &executable, &target)
}

fn build(opts: &TestFuzz, quiet: bool) -> Result<Vec<Executable>> {
    let metadata = MetadataCommand::new().exec()?;

    // smoelius: Put --message-format=json last so that it is easy to copy-and-paste the command
    // without it.
    let mut args = vec![];
    if !opts.no_instrumentation {
        args.extend_from_slice(&["afl"]);
    }
    args.extend_from_slice(&["test", "--frozen", "--no-run"]);
    let target_dir = target_directory(true);
    let target_dir_str = target_dir.to_string_lossy();
    if !opts.no_instrumentation {
        args.extend_from_slice(&["--target-dir", &target_dir_str]);
    }
    if let Some(package) = &opts.package {
        args.extend_from_slice(&["--package", package])
    }
    if opts.persistent {
        args.extend_from_slice(&["--features", "test-fuzz/persistent"]);
    }
    args.extend_from_slice(&["--message-format=json"]);

    // smoelius: Suppress "Warning: AFL++ tools will need to set AFL_MAP_SIZE..." Setting
    // `AFL_QUIET=1` doesn't work here, so pipe standard error to /dev/null.
    // smoelius: Suppressing all of standard error is too extreme. For now, suppress only when
    // displaying/replaying.
    let mut exec = Exec::cmd("cargo").args(&args).stdout(Redirection::Pipe);
    if quiet {
        exec = exec.stderr(NullFile);
    }
    debug!("{:?}", exec);
    let mut popen = exec.clone().popen()?;
    let messages = popen
        .stdout
        .take()
        .map_or(Ok(vec![]), |stream| -> Result<_> {
            let reader = BufReader::new(stream);
            let messages: Vec<Message> = Message::parse_stream(reader)
                .collect::<std::result::Result<_, std::io::Error>>()?;
            Ok(messages)
        })?;
    let status = popen.wait()?;

    ensure!(status.success(), "command failed: {:?}", exec);

    Ok(messages
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
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn test_fuzz_and_afl_versions(
    metadata: &Metadata,
    package_id: &PackageId,
) -> Result<(Option<Version>, Option<Version>)> {
    let test_fuzz = package_dependency(metadata, package_id, "test-fuzz")?;
    let afl = test_fuzz
        .as_ref()
        .map(|package_id| package_dependency(metadata, package_id, "afl"))
        .transpose()?
        .flatten();
    let test_fuzz_version = test_fuzz
        .map(|package_id| package_version(metadata, &package_id))
        .transpose()?;
    let afl_version = afl
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
    Ok(node
        .dependencies
        .iter()
        .map(|package_id| package_name(metadata, package_id).map(|s| (package_id, s == name)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .find_map(|(package_id, found)| {
            if found {
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
    let stream = exec.stream_stdout()?;

    // smoelius: A test executable's --list output ends with an empty line followed by
    // "M tests, N benchmarks." Stop at the empty line.
    let mut targets = Vec::<String>::default();
    for line in BufReader::new(stream).lines() {
        let line = line?;
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

fn executable_target(
    opts: &TestFuzz,
    executable_targets: &[(Executable, Vec<String>)],
) -> Result<(Executable, String)> {
    let mut executable_targets = executable_targets.to_vec();

    ensure!(
        !executable_targets.is_empty(),
        "found no fuzz targets{}",
        match_message(opts)
    );

    ensure!(
        executable_targets.len() <= 1,
        "found multiple executables with fuzz targets{}: {:#?}",
        match_message(opts),
        executable_targets
    );

    let mut executable_targets = executable_targets.remove(0);

    assert!(!executable_targets.1.is_empty());

    ensure!(
        executable_targets.1.len() <= 2,
        "found multiple fuzz targets{} in {:?}: {:#?}",
        match_message(opts),
        executable_targets.0,
        executable_targets.1
    );

    Ok((executable_targets.0, executable_targets.1.remove(0)))
}

fn match_message(opts: &TestFuzz) -> String {
    opts.target.as_ref().map_or("".to_owned(), |pat| {
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
    let cargo_afl_version = cargo_afl_version()?;
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
            &cargo_afl_version,
        )?;
    }
    Ok(())
}

fn cargo_afl_version() -> Result<Version> {
    let output = Command::new("cargo").args(&["afl", "--version"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.strip_prefix("cargo-afl ").ok_or_else(|| {
        anyhow!(
            "Could not determine `cargo-afl` version. Is it installed? Try `cargo install afl`."
        )
    })?;
    Version::parse(version).map_err(Into::into)
}

fn check_dependency_version(
    name: &str,
    dependency: &str,
    dependency_version: Option<&Version>,
    install: &str,
    install_version: &Version,
) -> Result<()> {
    if let Some(dependency_version) = dependency_version {
        ensure!(
            as_version_req(dependency_version).matches(install_version)
                || as_version_req(install_version).matches(dependency_version),
            "`{}` depends on `{} {}`, which is incompatible with `{} {}`.",
            name,
            dependency,
            dependency_version,
            install,
            install_version
        );
        if !as_version_req(dependency_version).matches(install_version) {
            eprintln!(
                "`{}` depends on `{} {}`, which is newer than `{} {}`. Consider upgrading with \
                `cargo install {} --force --version '>={}'`.",
                name,
                dependency,
                dependency_version,
                install,
                install_version,
                install,
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
    VersionReq::parse(&version.to_string()).expect("could not parse version as version request")
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
                for entry in read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .map(OsStr::to_string_lossy)
                        .unwrap_or_default();

                    if file_name == "README.txt" || file_name == ".state" {
                        continue;
                    }

                    let data = read(path)?;
                    test_fuzz::runtime::write_data(&corpus_dir, &data)?;
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
            remove_dir_all(output_dir).or_else(|err| {
                if format!("{}", err).starts_with("No such file or directory") {
                    Ok(())
                } else {
                    Err(err)
                }
            })?;
        }
    }

    Ok(())
}

fn for_each_entry(
    opts: &TestFuzz,
    executable: &Executable,
    target: &str,
    display: bool,
    replay: bool,
    dir: &Path,
) -> Result<()> {
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

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file = File::open(&path)?;
        let file_name = path
            .file_name()
            .map(OsStr::to_string_lossy)
            .unwrap_or_default();

        if file_name == "README.txt" || file_name == ".state" {
            continue;
        }

        let exec = Exec::cmd(&executable.path)
            .env_extend(&envs)
            .args(&args)
            .stdin(file)
            .stdout(NullFile)
            .stderr(Redirection::Pipe);
        debug!("{:?}", exec);
        let mut popen = exec.popen()?;
        let millis = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let time = Duration::from_millis(millis);
        let mut communicator = popen.communicate_start(None).limit_time(time);
        let (buffer, status) = match communicator.read() {
            Ok((_, buffer)) => {
                let status = popen.wait()?;
                (buffer.unwrap_or_default(), Some(status))
            }
            Err(CommunicateError {
                error,
                capture: (_, buffer),
            }) => {
                popen.kill()?;
                if error.kind() != std::io::ErrorKind::TimedOut {
                    return Err(anyhow!(error));
                }
                let _ = popen.wait()?;
                (buffer.unwrap_or_default(), None)
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
        if let Some(status) = status {
            if buffer.is_empty() {
                println!("{:?}", status);
            }
            failure |= !status.success();
        } else {
            println!("Timeout");
            timeout = true;
        }

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
    let corpus_dir = corpus_directory_from_target(&executable.name, target)
        .to_string_lossy()
        .into_owned();

    let output_dir = output_directory_from_target(&executable.name, target);
    create_dir_all(&output_dir).unwrap_or_default();

    let mut command = Command::new("cargo");

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
            if opts.resume { "-" } else { &corpus_dir },
            "-o",
            &output_dir.to_string_lossy(),
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
    args.extend(opts.args.clone());
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

    command.envs(envs).args(args);

    let status = command.status()?;

    ensure!(status.success(), "command failed: {:?}", command);

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::cargo_test_fuzz as cargo;
    use anyhow::Result;
    use lazy_static::lazy_static;
    use std::{env, io};

    const TEST_DIR: &str = "../examples";

    lazy_static! {
        static ref INITIALIZE: io::Result<()> = {
            // println!("{:?}", env::current_dir()?);
            env::set_current_dir(TEST_DIR)
        };
    }

    #[test]
    fn build_no_instrumentation_with_target() {
        INITIALIZE.as_ref().unwrap();
        cargo_test_fuzz(&["--no-run", "--no-instrumentation", "--target", "target"]).unwrap();
    }

    fn cargo_test_fuzz(args: &[&str]) -> Result<()> {
        let mut cargo_args = vec!["cargo-test-fuzz", "test-fuzz"];
        cargo_args.extend_from_slice(args);
        cargo(&cargo_args)
    }
}
