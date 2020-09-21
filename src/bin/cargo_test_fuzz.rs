#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::panic)]

use anyhow::{ensure, Result};
use cargo_metadata::{Artifact, ArtifactProfile, Message};
use clap::Clap;
use dirs::{corpus_directory_from_target, output_directory_from_target};
use log::debug;
use std::{
    fmt::Debug,
    fs::create_dir_all,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};
use subprocess::Exec;

const ENTRY_SUFFIX: &str = "_fuzz::entry";

#[derive(Clap, Debug)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    TestFuzz(TestFuzz),
}

#[derive(Clap, Debug)]
struct TestFuzz {
    #[clap(long, about = "List fuzz targets")]
    list: bool,
    #[clap(long, about = "Resume last fuzzing session")]
    resume: bool,
    #[clap(long, about = "For testing build process")]
    no_instrumentation: bool,
    #[clap(short, long, about = "Package containing fuzz target")]
    package: Option<String>,
    #[clap(long, about = "String that fuzz target's name must contain")]
    target: Option<String>,
    #[clap(last = true, about = "Arguments for the fuzzer")]
    args: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();

    let SubCommand::TestFuzz(opts) = Opts::parse().subcmd;

    let executables = build(&opts)?;

    let mut executable_targets = executable_targets(&executables)?;

    if let Some(pat) = &opts.target {
        executable_targets = filter_executable_targets(&pat, &executable_targets);
    }

    if opts.list {
        println!("{:#?}", executable_targets);
        return Ok(());
    }

    let (executable, krate, target) = executable_target(&opts, &executable_targets)?;

    if opts.no_instrumentation {
        println!("Stopping before fuzzing since --no-instrumentation was specified.");
        return Ok(());
    }

    fuzz(&opts, &executable, &krate, &target)
}

fn build(opts: &TestFuzz) -> Result<Vec<(PathBuf, String)>> {
    let mut args = vec![];
    if !opts.no_instrumentation {
        args.extend_from_slice(&["afl"]);
    }
    args.extend_from_slice(&["test", "--no-run", "--message-format=json"]);
    if let Some(package) = &opts.package {
        args.extend_from_slice(&["--package", &package])
    }

    let exec = Exec::cmd("cargo").args(&args);
    debug!("{:?}", exec);
    let stream = exec.stream_stdout()?;
    let reader = BufReader::new(stream);

    let messages: Vec<Message> =
        Message::parse_stream(reader).collect::<std::result::Result<_, std::io::Error>>()?;

    Ok(messages
        .into_iter()
        .filter_map(|message| {
            if let Message::CompilerArtifact(Artifact {
                target: build_target,
                profile: ArtifactProfile { test: true, .. },
                executable: Some(executable),
                ..
            }) = message
            {
                Some((executable, build_target.name))
            } else {
                None
            }
        })
        .collect())
}

fn executable_targets(
    executables: &[(PathBuf, String)],
) -> Result<Vec<(PathBuf, String, Vec<String>)>> {
    let executable_targets: Vec<(PathBuf, String, Vec<String>)> = executables
        .iter()
        .map(|(executable, krate)| {
            let targets = targets(executable)?;
            Ok((executable.clone(), krate.clone(), targets))
        })
        .collect::<Result<_>>()?;

    Ok(executable_targets
        .into_iter()
        .filter(|executable_targets| !executable_targets.2.is_empty())
        .collect())
}

fn targets(executable: &PathBuf) -> Result<Vec<String>> {
    let exec = Exec::cmd(executable).args(&["--list"]);
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

fn filter_executable_targets(
    pat: &str,
    executable_targets: &[(PathBuf, String, Vec<String>)],
) -> Vec<(PathBuf, String, Vec<String>)> {
    executable_targets
        .iter()
        .filter_map(|(executable, krate, targets)| {
            let targets = filter_targets(pat, targets);
            if !targets.is_empty() {
                Some((executable.clone(), krate.clone(), targets))
            } else {
                None
            }
        })
        .collect()
}

fn filter_targets(pat: &str, targets: &[String]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| target.contains(pat))
        .cloned()
        .collect()
}

fn executable_target(
    opts: &TestFuzz,
    executable_targets: &[(PathBuf, String, Vec<String>)],
) -> Result<(PathBuf, String, String)> {
    let mut executable_targets = executable_targets.to_vec();

    ensure!(!executable_targets.is_empty(), "found no fuzz targets");

    ensure!(
        executable_targets.len() <= 1,
        "found multiple executables with fuzz targets{}: {:#?}",
        opts.target
            .as_ref()
            .map_or("".to_owned(), |pat| format!(" matching `{}`", pat)),
        executable_targets
    );

    let mut executable_targets = executable_targets.remove(0);

    assert!(!executable_targets.2.is_empty());

    ensure!(
        executable_targets.2.len() <= 2,
        "found multiple fuzz targets{} in {:?}: {:#?}",
        opts.target
            .as_ref()
            .map_or("".to_owned(), |pat| format!(" matching `{}`", pat)),
        (executable_targets.0, executable_targets.1),
        executable_targets.2
    );

    Ok((
        executable_targets.0,
        executable_targets.1,
        executable_targets.2.remove(0),
    ))
}

fn fuzz(opts: &TestFuzz, executable: &PathBuf, krate: &str, target: &str) -> Result<()> {
    let corpus = corpus_directory_from_target(krate, target)
        .to_string_lossy()
        .into_owned();

    let output = output_directory_from_target(krate, target);
    create_dir_all(&output).unwrap_or_default();

    let mut command = Command::new("cargo");

    command.env("TEST_FUZZ", "1");

    let mut args = vec![];
    args.extend(
        vec![
            "afl",
            "fuzz",
            "-i",
            if opts.resume { "-" } else { &corpus },
            "-o",
            &output.to_string_lossy(),
        ]
        .into_iter()
        .map(String::from),
    );
    args.extend(opts.args.clone());
    args.extend(
        vec![
            "--",
            &executable.to_string_lossy(),
            "--exact",
            &(target.to_owned() + ENTRY_SUFFIX),
        ]
        .into_iter()
        .map(String::from),
    );

    command.args(args);

    let status = command.status()?;

    ensure!(status.success(), "command failed: {:?}", command);

    Ok(())
}
