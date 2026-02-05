use anyhow::{Context, Result, bail, ensure};
use cargo_metadata::{Artifact, ArtifactProfile, Message};
use internal::serde_format;
use log::debug;
use std::process::Command;
use subprocess::{Exec, Redirection};

#[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
pub const MANIFEST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../fuzzable/Cargo.toml");

// smoelius: We want to reuse the existing features. So we can't do anything that would cause the
// fuzzable examples to be rebuilt.
// smoelius: What was I worried about? What would cause the fuzzable examples to be rebuilt?
// Something about the current working directory? Maybe using the manifest path solves that problem?
pub fn test(krate: &str, test: &str) -> Result<Command> {
    // smoelius: Put --message-format=json last so that it is easy to copy-and-paste the command
    // without it.
    let serde_format_feature = "test-fuzz/".to_owned() + serde_format::as_feature();
    let mut args = vec![
        "test",
        "--manifest-path",
        MANIFEST_PATH,
        "--test",
        krate,
        "--features",
        &serde_format_feature,
    ];
    args.extend_from_slice(&["--no-run", "--message-format=json"]);

    // smoelius: As mentioned in cargo-test-fuzz/src/lib.rs, `LLVM_PROFILE_FILE` must be set every
    // time a binary with coverage instrumentation is run. However, there is no easy way to tell
    // whether a test binary was compiled with coverage instrumentation. So it is unclear whether
    // `LLVM_PROFILE_FILE` should be set here.
    #[allow(clippy::disallowed_methods, reason = "runs `cargo test`")]
    let exec = Exec::cmd("cargo").args(&args).stdout(Redirection::Pipe);
    debug!("{exec:?}");
    let mut popen = exec.clone().popen()?;
    let messages = popen
        .stdout
        .take()
        .map_or(Ok(vec![]), |stream| -> Result<_> {
            let reader = std::io::BufReader::new(stream);
            let messages: Vec<Message> = Message::parse_stream(reader)
                .collect::<std::result::Result<_, std::io::Error>>()
                .with_context(|| format!("`parse_stream` failed for `{exec:?}`"))?;
            Ok(messages)
        })?;
    let status = popen
        .wait()
        .with_context(|| format!("`wait` failed for `{popen:?}`"))?;

    ensure!(status.success(), "Command failed: {exec:?}");

    let executables = messages
        .into_iter()
        .filter_map(|message| {
            if let Message::CompilerArtifact(Artifact {
                profile: ArtifactProfile { test: true, .. },
                executable: Some(executable),
                ..
            }) = message
                && let Some(file_name) = executable.file_name()
                && file_name.starts_with(&(krate.to_owned() + "-"))
            {
                Some(executable)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ensure!(
        executables.len() <= 1,
        "Found multiple executables starting with `{krate}`"
    );

    if let Some(executable) = executables.into_iter().next() {
        #[allow(clippy::disallowed_methods, reason = "runs `cargo test-fuzz`")]
        let mut command = Command::new(executable);
        command.env("TEST_FUZZ_ID", id());
        command.args(["--exact", test]);
        Ok(command)
    } else {
        bail!("Found no executables starting with `{krate}`")
    }
}

pub fn test_fuzz_all() -> Result<Command> {
    let serde_format_feature = "test-fuzz/".to_owned() + serde_format::as_feature();
    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let args = vec![
        "run",
        "--bin=cargo-test-fuzz",
        "--manifest-path",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"),
        "--",
        "test-fuzz",
        "--manifest-path",
        MANIFEST_PATH,
        "--features",
        &serde_format_feature,
    ];

    #[allow(clippy::disallowed_methods, reason = "runs `cargo test-fuzz`")]
    let mut command = Command::new("cargo");
    command.env("AFL_NO_AFFINITY", "1");
    command.env("TEST_FUZZ_ID", id());
    command.args(args);
    Ok(command)
}

pub fn test_fuzz(krate: &str, target: &str) -> Result<Command> {
    test_fuzz_all().map(|mut command| {
        command.args(["--test", krate, "--exact", target]);
        command
    })
}

pub fn test_fuzz_inexact(krate: &str, target: &str) -> Result<Command> {
    test_fuzz_all().map(|mut command| {
        command.args(["--test", krate, target]);
        command
    })
}

fn id() -> String {
    std::env::var("TEST_FUZZ_ID").unwrap_or_else(|_| thread_id())
}

fn thread_id() -> String {
    format!("{:?}", std::thread::current().id()).replace(['(', ')'], "_")
}
