#![cfg_attr(dylint_lib = "general", allow(crate_wide_allow))]
// smoelius: All work is done in temporary directories. So there are no races.
#![cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#![cfg_attr(dylint_lib = "try_io_result", allow(try_io_result))]

use anyhow::Result;
use std::{env::join_paths, fs::write, path::Path};
use tempfile::NamedTempFile;
use xshell::{Shell, cmd};

// Verify that cargo-test-fuzz reports an error when cargo-afl is not installed.
#[test]
fn uninstalled_cargo_afl() -> Result<()> {
    run_test(
        None,
        &[],
        &[
            "\\<Could not determine `cargo-afl` version. Is it installed? Try `cargo install \
             cargo-afl`.$",
        ],
    )
}

// Verify that cargo-test-fuzz reports an error when the selected afl package is incompatible with
// the installed cargo-afl version.
#[ignore = "reenable once a version of cargo-afl with rustix@1.1.5 has been published"]
#[test]
fn incompatible_cargo_afl() -> Result<()> {
    run_test(
        Some("0.13.2"),
        &[],
        &[
            "^Error: `[^`]*` depends on `afl [^`]*`, which is incompatible with `cargo-afl \
             [^`]*`.$",
        ],
    )
}

// Verify that cargo-test-fuzz warns and suggests an upgrade when the selected afl package is newer
// than the installed cargo-afl version.
#[test]
#[ignore = "reenable once two versions of cargo-afl with rustix@1.1.5 have been published"]
fn newer_afl() -> Result<()> {
    run_test(
        Some("0.13.2"),
        &[(
            r#"s/^\(afl = {.*\<version = "\)[^"]*\(".*}\)$/\1=0.13.3\2/"#,
            &["Cargo.toml"],
        )],
        &[
            "^`[^`]*` depends on `afl [^`]*`, which is newer than `cargo-afl [^`]*`.",
            "Consider upgrading with `cargo install cargo-afl --force --version [^`]*`.$",
        ],
    )
}

// Verify that cargo-test-fuzz reports an error when the selected test-fuzz package has an
// incompatible version.
#[test]
fn incompatible_test_fuzz() -> Result<()> {
    let mut version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
    version.major = 0;
    version.minor = 0;
    let version_script =
        format!(r#"s/^version = {{ workspace = true }}$/version = \"{version}\"/"#);

    run_test(
        Some("*"),
        &[
            (version_script.as_str(), &["test-fuzz/Cargo.toml"]),
            (
                r#"s/^\(test-fuzz = {.*\<version = "=\)[^.]*\.[^.]*\.\([^"]*".*}\)$/\10.0.\2/"#,
                &["Cargo.toml"],
            ),
        ],
        &[
            "^Error: `[^`]*` depends on `test-fuzz [^`]*`, which is incompatible with \
             `cargo-test-fuzz [^`]*`.$",
        ],
    )
}

// Verify that cargo-test-fuzz warns and suggests an upgrade when the selected test-fuzz package is
// newer than cargo-test-fuzz.
#[test]
fn newer_test_fuzz() -> Result<()> {
    let mut test_fuzz_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
    test_fuzz_version.patch = 255;
    test_fuzz_version.pre = semver::Prerelease::EMPTY;
    test_fuzz_version.build = semver::BuildMetadata::EMPTY;
    let test_fuzz_version_script =
        format!(r#"s/^version = {{ workspace = true }}$/version = \"{test_fuzz_version}\"/"#);

    let cargo_test_fuzz_version = env!("CARGO_PKG_VERSION")
        .split_once('-')
        .map_or(env!("CARGO_PKG_VERSION"), |(version, _)| version);
    let cargo_test_fuzz_version_script =
        format!(r#"s/^version = {{ workspace = true }}$/version = \"{cargo_test_fuzz_version}\"/"#);

    run_test(
        Some("*"),
        &[
            (test_fuzz_version_script.as_str(), &["test-fuzz/Cargo.toml"]),
            (
                r#"s/^\(test-fuzz = {.*\<version = "=[^.]*\.[^.]*\)\.[^"]*\(".*}\)$/\1.255\2/"#,
                &["Cargo.toml"],
            ),
            (
                cargo_test_fuzz_version_script.as_str(),
                &["cargo-test-fuzz/Cargo.toml"],
            ),
        ],
        &[
            "^`[^`]*` depends on `test-fuzz [^`]*`, which is newer than `cargo-test-fuzz [^`]*`.",
            "Consider upgrading with `cargo install cargo-test-fuzz --force --version [^`]*`.$",
        ],
    )
}

fn run_test(
    afl_version: Option<&str>,
    sed_scripts_and_paths: &[(&str, &[&str])],
    grep_patterns: &[&str],
) -> Result<()> {
    sandbox(|sh| {
        if let Some(version) = afl_version {
            cmd!(sh, "cargo install cargo-afl --version={version} --quiet").run()?;
        }

        for &(script, paths) in sed_scripts_and_paths {
            cmd!(sh, "sed -i {script} {paths...}").run()?;
        }

        let output = cmd!(
            sh,
            "cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-fuzzable --no-run"
        )
        .ignore_status()
        .read_stderr()?;

        println!("{output}");

        let tempfile = write_to_tempfile(&output)?;
        let tempfile_path = tempfile.path();

        for pattern in grep_patterns {
            cmd!(sh, "grep -m 1 {pattern} {tempfile_path}").run()?;
        }

        Ok(())
    })
}

fn sandbox(f: impl FnOnce(Shell) -> Result<()>) -> Result<()> {
    let sh = Shell::new()?;

    let home = sh.create_temp_dir()?;
    sh.change_dir(home.path());

    sh.set_var("PATH", "/usr/bin");

    install_rust(&sh)?;

    // smoelius: `HOME` can be set only after Rust is installed. See, e.g.:
    // https://github.com/rust-lang/rustup/issues/1884#issuecomment-498157692
    sh.set_var("HOME", home.path());

    #[cfg_attr(dylint_lib = "general", allow(abs_home_path))]
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");

    cmd!(sh, "git clone {repo} workdir").run()?;

    sh.change_dir("workdir");

    f(sh)
}

fn install_rust(sh: &Shell) -> Result<()> {
    let cargo_home = sh.current_dir().join(".cargo");
    let rustup_home = sh.current_dir().join(".rustup");

    sh.set_var("CARGO_HOME", &cargo_home);
    sh.set_var("RUSTUP_HOME", rustup_home);

    let output = cmd!(
        sh,
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs"
    )
    .read()?;

    let tempfile = write_to_tempfile(&output)?;
    let tempfile_path = tempfile.path();

    cmd!(
        sh,
        "sh {tempfile_path} -y --no-modify-path --profile minimal --quiet"
    )
    .run()?;

    let path_old = sh.var("PATH")?;
    let path_new = join_paths([format!("{}/bin", cargo_home.to_string_lossy()), path_old])?;
    sh.set_var("PATH", path_new);

    Ok(())
}

fn write_to_tempfile(contents: &str) -> Result<NamedTempFile> {
    let tempfile = NamedTempFile::new()?;
    write(&tempfile, contents)?;
    Ok(tempfile)
}
