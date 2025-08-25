use anyhow::{Context, Result, bail, ensure};
use assert_cmd::cargo::CommandCargoExt;
use cargo_metadata::{Artifact, ArtifactProfile, Message};
use internal::serde_format;
use log::debug;
use std::{path::Path, process::Command, sync::LazyLock};
use subprocess::{Exec, Redirection};

pub static MANIFEST_PATH: LazyLock<String> = LazyLock::new(|| {
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

    ensure!(status.success(), "Command failed: {:?}", exec);

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
        "Found multiple executables starting with `{}`",
        krate
    );

    if let Some(executable) = executables.into_iter().next() {
        let mut command = Command::new(executable);
        command.env("TEST_FUZZ_ID", id());
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
