use anyhow::{bail, ensure, Context, Result};
use assert_cmd::Command;
use cargo_metadata::{Artifact, ArtifactProfile, Message};
use if_chain::if_chain;
use internal::serde_format;
use log::debug;
use once_cell::sync::Lazy;
use std::{
    path::Path,
    process::{Command as StdCommand, Stdio},
};

pub static MANIFEST_PATH: Lazy<String> = Lazy::new(|| {
    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/Cargo.toml")
        .to_string_lossy()
        .to_string()
});

// smoelius: We want to reuse the existing features. So we can't do anything that would cause the
// examples to be rebuilt.
// smoelius: What was I worried about? What would cause the examples to be rebuilt? Something about
// the current working directory? Maybe using the manifest path solves that problem?
pub fn test(krate: &str, test: &str) -> Result<Command> {
    // smoelius: Put --message-format=json last so that it is easy to copy-and-paste the command
    // without it.
    let serde_format_feature = "test-fuzz/".to_owned() + serde_format::as_feature();
    let mut args = vec![
        "test",
        "--manifest-path",
        &*MANIFEST_PATH,
        "--test",
        krate,
        "--features",
        &serde_format_feature,
    ];
    args.extend_from_slice(&["--no-run", "--message-format=json"]);

    let mut command = StdCommand::new("cargo");
    command.args(&args).stdout(Stdio::piped());
    debug!("{command:?}");
    let exec = format!("{command:?}");
    let mut popen = command
        .spawn()
        .with_context(|| format!("Could not spawn `{exec:?}`"))?;
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

    ensure!(status.success(), "Command failed: {:?}", exec);

    let executables = messages
        .into_iter()
        .filter_map(|message| {
            if_chain! {
                if let Message::CompilerArtifact(Artifact {
                    profile: ArtifactProfile { test: true, .. },
                    executable: Some(executable),
                    ..
                }) = message;
                if let Some(file_name) = executable.file_name();
                if file_name.starts_with(&(krate.to_owned() + "-"));
                then {
                    Some(executable)
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    ensure!(
        executables.len() <= 1,
        "Found multiple executables starting with `{}`",
        krate
    );

    if let Some(executable) = executables.into_iter().next() {
        let mut command = Command::new(executable);
        command.args(["--exact", test]);
        Ok(command)
    } else {
        bail!("Found no executables starting with `{}`", krate)
    }
}

pub fn test_fuzz_all() -> Result<Command> {
    let serde_format_feature = "test-fuzz/".to_owned() + serde_format::as_feature();
    let args = vec![
        "test-fuzz",
        "--manifest-path",
        &*MANIFEST_PATH,
        "--features",
        &serde_format_feature,
    ];

    let mut command = Command::cargo_bin("cargo-test-fuzz")?;
    command.args(&args);
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
