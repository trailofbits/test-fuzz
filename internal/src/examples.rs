use anyhow::{bail, ensure, Context, Result};
use assert_cmd::Command;
use cargo_metadata::{Artifact, ArtifactProfile, Message, MetadataCommand};
use if_chain::if_chain;
use log::debug;
use std::io::BufReader;
use subprocess::{Exec, Redirection};

// smoelius: We want to reuse the existing features. So we can't do anything that would cause the
// examples to be rebuilt.
pub fn test(krate: &str, test: &str) -> Result<Command> {
    let metadata = MetadataCommand::new().exec()?;

    // smoelius: Put --message-format=json last so that it is easy to copy-and-paste the command
    // without it.
    let args = [
        "test",
        "--package",
        "test-fuzz-examples",
        "--no-default-features",
        "--features",
        &("test-fuzz/serde_".to_owned() + crate::serde_format()),
        "--no-run",
        "--message-format=json",
    ];

    let exec = Exec::cmd("cargo")
        .cwd(metadata.workspace_root)
        .args(&args)
        .stdout(Redirection::Pipe);
    debug!("{:?}", exec);
    let mut popen = exec.clone().popen()?;
    let messages = popen
        .stdout
        .take()
        .map_or(Ok(vec![]), |stream| -> Result<_> {
            let reader = BufReader::new(stream);
            let messages: Vec<Message> = Message::parse_stream(reader)
                .collect::<std::result::Result<_, std::io::Error>>()
                .with_context(|| format!("`parse_stream` failed for `{:?}`", exec))?;
            Ok(messages)
        })?;
    let status = popen
        .wait()
        .with_context(|| format!("`wait` failed for `{:?}`", popen))?;

    ensure!(status.success(), "Command failed: {:?}", exec);

    let messages = messages
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
        messages.len() <= 1,
        "Found multiple executables starting with `{}`",
        krate
    );

    if let Some(message) = messages.into_iter().next() {
        let mut cmd = Command::new(message);
        cmd.args(&["--exact", test]);
        Ok(cmd)
    } else {
        bail!("Found no executables starting with `{}`", krate)
    }
}

pub fn test_fuzz(target: &str) -> Result<Command> {
    let mut cmd = Command::cargo_bin("cargo-test-fuzz")?;
    cmd.args(&[
        "test-fuzz",
        "--package",
        "test-fuzz-examples",
        "--no-default-features",
        "--features",
        &("test-fuzz/serde_".to_owned() + crate::serde_format()),
        "--exact",
        "--target",
        target,
    ]);
    Ok(cmd)
}
