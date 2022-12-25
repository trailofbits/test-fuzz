use super::super::Interface;
use crate::{
    check_dependency_version, exec_from_command, Executable, TestFuzz, BASE_ENVS, ENTRY_SUFFIX,
    MILLIS_PER_SEC,
};
use anyhow::{anyhow, bail, ensure, Context, Result};
use internal::Fuzzer;
use log::debug;
use once_cell::sync::OnceCell;
use semver::Version;
use std::{
    io::BufRead,
    path::Path,
    process::{exit, Command},
};
use subprocess::Redirection;

struct Impl {
    persistent: bool,
}

static IMPL_PERSISTENT: Impl = Impl { persistent: true };
static IMPL: Impl = Impl { persistent: false };

pub(crate) fn instantiate(persistent: bool) -> &'static dyn Interface {
    if persistent {
        &IMPL_PERSISTENT
    } else {
        &IMPL
    }
}

impl Interface for Impl {
    fn to_enum(&self) -> internal::Fuzzer {
        if self.persistent {
            Fuzzer::AflplusplusPersistent
        } else {
            Fuzzer::Aflplusplus
        }
    }

    fn dependency_name(&self) -> Option<&'static str> {
        if self.persistent {
            Some("afl")
        } else {
            None
        }
    }

    fn check_dependency_version(&self, executable_name: &str, version: &Version) -> Result<()> {
        check_dependency_version(
            executable_name,
            "afl",
            Some(version),
            "cargo-afl",
            cached_cargo_afl_version(),
            "afl",
        )
    }

    fn build(&self, manifest_path: Option<&Path>) -> Result<Command> {
        let mut command = Self::make_cargo_command(manifest_path, "test");
        command.args(["--no-run"]);
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
        let mut command = Command::new("cargo");

        command.envs(BASE_ENVS.iter().copied());
        if opts.no_ui {
            command.env("AFL_NO_UI", "1");
        }
        if opts.run_until_crash {
            command.env("AFL_BENCH_UNTIL_CRASH", "1");
        }

        command.args([
            "afl",
            "fuzz",
            "-i",
            &input_dir.to_string_lossy(),
            "-o",
            &output_dir.to_string_lossy(),
            "-D",
            "-M",
            "default",
        ]);
        if let Some(max_total_time) = opts.max_total_time {
            command.args(["-V".to_owned(), max_total_time.to_string()]);
        }
        if let Some(timeout) = opts.timeout {
            command.args(["-t".to_owned(), (timeout * MILLIS_PER_SEC).to_string()]);
        }
        command.args(opts.zzargs.clone());
        command.args([
            "--",
            &executable.path.to_string_lossy(),
            "--exact",
            &(target.to_owned() + ENTRY_SUFFIX),
        ]);

        #[allow(clippy::if_not_else)]
        if !opts.exit_code {
            debug!("{:?}", command);
            let status = command
                .status()
                .with_context(|| format!("Could not get status of `{command:?}`"))?;

            ensure!(status.success(), "Command failed: {:?}", command);
        } else {
            let exec = exec_from_command(&command).stdout(Redirection::Pipe);
            debug!("{:?}", exec);
            let mut popen = exec.clone().popen()?;
            let stdout = popen
                .stdout
                .take()
                .ok_or_else(|| anyhow!("Could not get output of `{:?}`", exec))?;
            let mut time_limit_was_reached = false;
            let mut testing_aborted_programmatically = false;
            for line in std::io::BufReader::new(stdout).lines() {
                let line = line.with_context(|| format!("Could not get output of `{exec:?}`"))?;
                if line.contains("Time limit was reached") {
                    time_limit_was_reached = true;
                }
                // smoelius: Work around "pizza mode" bug.
                if line.contains("+++ Testing aborted programmatically +++")
                    || line.contains("+++ Baking aborted programmatically +++")
                {
                    testing_aborted_programmatically = true;
                }
                println!("{line}");
            }
            let status = popen
                .wait()
                .with_context(|| format!("`wait` failed for `{popen:?}`"))?;

            if !testing_aborted_programmatically || !status.success() {
                bail!("Command failed: {:?}", exec);
            }

            if !time_limit_was_reached {
                exit(1);
            }
        }

        Ok(())
    }
}

impl Impl {
    fn make_cargo_command(manifest_path: Option<&Path>, subcommand: &str) -> Command {
        // smoelius: Ensure `cargo-afl` is installed.
        let _ = cached_cargo_afl_version();

        let mut command = Command::new("cargo");
        command.args(["afl", subcommand]);
        if let Some(path) = manifest_path {
            command.args(["--manifest-path", &path.to_string_lossy()]);
        }
        command
    }
}

fn cached_cargo_afl_version() -> &'static Version {
    #[allow(clippy::unwrap_used)]
    CARGO_AFL_VERSION.get_or_init(|| cargo_afl_version().unwrap())
}

static CARGO_AFL_VERSION: OnceCell<Version> = OnceCell::new();

fn cargo_afl_version() -> Result<Version> {
    let mut command = Command::new("cargo");
    command.args(["afl", "--version"]);
    let output = command
        .output()
        .with_context(|| format!("Could not get output of `{command:?}`"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.strip_prefix("cargo-afl ").ok_or_else(|| {
        anyhow!(
            "Could not determine `cargo-afl` version. Is it installed? Try `cargo install afl`."
        )
    })?;
    Version::parse(version.trim_end()).map_err(Into::into)
}
