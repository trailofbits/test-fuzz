use super::super::Interface;
use crate::{Executable, TestFuzz, BASE_ENVS, ENTRY_SUFFIX};
use anyhow::{bail, Context, Result};
use cargo_fuzz::{
    options::{default_opts, BuildOptions, Sanitizer},
    project::FuzzProject,
};
use internal::Fuzzer;
use log::debug;
use semver::Version;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
    process::{exit, Command},
    time::Duration,
};

const OUTPUT_GRACE: u64 = 25; // minutes

// smoelius: libfuzzer's default is 20 minutes, which is way too long. 1000 milliseconds is
// AFLplusplus's default timeout:
// https://github.com/AFLplusplus/AFLplusplus/blob/0c0a6c3bfabf0facaed33fae1aa5ad54a6a11b32/include/config.h#L127-L130
const DEFAULT_TIMEOUT: u64 = 1000; // milliseconds

// smoelius:
// https://github.com/rust-fuzz/libfuzzer/blob/03a00b2c2ab7b82838536883e0b66eae59ab130d/libfuzzer/FuzzerOptions.h#L23-L26
// `INTERRUPT_EXIT_CODE` is currently unused.
const TIMEOUT_EXIT_CODE: i32 = 70;
const OOM_EXIT_CODE: i32 = 71;
#[allow(dead_code)]
const INTERRUPT_EXIT_CODE: i32 = 72;
const ERROR_EXIT_CODE: i32 = 77;

struct Impl;

static IMPL: Impl = Impl;

pub(crate) fn instantiate() -> &'static dyn Interface {
    &IMPL
}

impl Interface for Impl {
    fn to_enum(&self) -> internal::Fuzzer {
        Fuzzer::Libfuzzer
    }

    fn dependency_name(&self) -> Option<&'static str> {
        Some("libfuzzer-sys")
    }

    fn check_dependency_version(&self, _executable_name: &str, _version: &Version) -> Result<()> {
        Ok(())
    }

    fn build(&self, manifest_path: Option<&Path>) -> Result<Command> {
        let project = Self::make_project(manifest_path)?;
        let build = BuildOptions {
            dev: true,
            release: false,
            sanitizer: Sanitizer::None,
            ..default_opts()
        };
        let mut command = project.cargo("build", &build)?;
        extend_env(&mut command, "RUSTFLAGS", " -C debuginfo=2");
        Ok(command)
    }

    fn fuzz(
        &self,
        opts: &TestFuzz,
        executable: &Executable,
        target: &str,
        input_dir: &Path,
        output_dir: &Path,
    ) -> Result<()> {
        if !opts.resume && output_dir.exists() {
            if !within_grace_period(output_dir)? {
                bail!("Found at-risk data in {output_dir:?}");
            }
            remove_dir_all(output_dir)
                .with_context(|| format!("Could not remove {output_dir:?}"))?;
        }

        let output_queue = output_dir.join("queue");
        let output_artifacts = output_dir.join("artifacts");

        create_dir_all(&output_queue).unwrap_or_default();
        create_dir_all(&output_artifacts).unwrap_or_default();

        fs_extra::dir::copy(
            input_dir,
            &output_queue,
            &fs_extra::dir::CopyOptions {
                content_only: true,
                overwrite: true,
                ..Default::default()
            },
        )?;

        let mut args = vec![
            output_queue.to_string_lossy().to_string(),
            format!("-artifact_prefix={}/", output_artifacts.to_string_lossy()),
            "-detect_leaks=0".to_owned(),
            // smoelius: Without `-reduce_inputs`, Libfuzzer fails to find hangs in
            // `parse_duration`. I don't know why.
            // "-reduce_inputs=0".to_owned(),
        ];
        if let Some(max_total_time) = opts.max_total_time {
            args.push(format!("-max_total_time={max_total_time}"));
        }
        let timeout = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
        args.push(format!("-timeout={}", (timeout + 999) / 1000));
        args.extend(opts.zzargs.iter().cloned());
        let args_json = serde_json::to_string(&args)?;

        let mut command = Command::new(&executable.path);

        command.envs(BASE_ENVS.iter().copied());

        command.env("TEST_FUZZ_LIBFUZZER_ARGS", args_json);

        command.args([
            "--exact",
            &(target.to_owned() + ENTRY_SUFFIX),
            "--nocapture",
            "--test-threads=1",
        ]);

        debug!("{:?}", command.get_envs());
        debug!("{:?}", command);

        let status = command
            .status()
            .with_context(|| format!("Could not get status of `{command:?}`"))?;

        if !status.success() {
            let code = status
                .code()
                .with_context(|| format!("Could not get exit code of `{command:?}`"))?;
            if opts.exit_code {
                exit(
                    if [TIMEOUT_EXIT_CODE, OOM_EXIT_CODE, ERROR_EXIT_CODE].contains(&code) {
                        1
                    } else {
                        2
                    },
                );
            }
            exit(code);
        }

        Ok(())
    }
}

impl Impl {
    fn make_project(manifest_path: Option<&Path>) -> Result<FuzzProject> {
        let manifest_dir = manifest_path
            .and_then(Path::parent)
            .or_else(|| Some(Path::new(".")));
        FuzzProject::new(manifest_dir.map(Path::to_path_buf))
    }
}

fn within_grace_period(path: &Path) -> Result<bool> {
    let metadata = path
        .metadata()
        .with_context(|| format!("Could not get {path:?} metadata"))?;
    let created = metadata
        .created()
        .with_context(|| format!("Could not get {path:?} creation time"))?;
    let modified = metadata
        .modified()
        .with_context(|| format!("Could not get {path:?} modification time"))?;
    let elapsed = modified.duration_since(created)?;
    Ok(elapsed < Duration::from_secs(OUTPUT_GRACE * 60))
}

fn extend_env(command: &mut Command, key: &str, val: &str) {
    let Some(mut val_next) = command
        .get_envs()
        .find_map(|(key_curr, val_curr)| if key == key_curr { val_curr } else { None })
        .map(ToOwned::to_owned)
    else {
        return;
    };
    val_next.push(val);
    command.env(key, val_next);
}
