#![cfg_attr(dylint_lib = "crate_wide_allow", allow(crate_wide_allow))]
// smoelius: All work is done in temporary directories. So there are no races.
#![cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#![cfg_attr(dylint_lib = "try_io_result", allow(try_io_result))]

use anyhow::Result;
use std::{env::join_paths, fs::write, path::Path};
use tempfile::NamedTempFile;
use xshell::{cmd, Shell};

/*
  test-uninstalled-cargo-afl:
    ...
      - name: Test
        run: |
          OUTPUT="$(cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run 2>&1 1>/dev/null || true)"
          echo "$OUTPUT"
          echo "$OUTPUT" | grep '^Error: Could not determine `cargo-afl` version. Is it installed? Try `cargo install afl`.$'
*/
#[test]
fn uninstalled_cargo_afl() -> Result<()> {
    run_test(
        None,
        &[],
        &[
            "^Error: Could not determine `cargo-afl` version. Is it installed? Try `cargo install \
             afl`.$",
        ],
    )
}

/*
  test-incompatible-cargo-afl:
    ...
      - name: Install older afl
        run: cargo install afl --version=0.11.0

      - name: Test
        run: |
          OUTPUT="$(cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run 2>&1 1>/dev/null || true)"
          echo "$OUTPUT"
          echo "$OUTPUT" | grep '^Error: `[^`]*` depends on `afl [^`]*`, which is incompatible with `cargo-afl [^`]*`.$'
*/
#[test]
fn incompatible_cargo_afl() -> Result<()> {
    run_test(
        Some("0.11.0"),
        &[],
        &[
            "^Error: `[^`]*` depends on `afl [^`]*`, which is incompatible with `cargo-afl \
             [^`]*`.$",
        ],
    )
}

/*
  test-newer-afl:
    ...
      - name: Install afl 0.12.2
        run: cargo install afl --version=0.12.2

      - name: Require afl 0.12.3
        run: |
          sed -i 's/^\(afl = {.*\<version = "\)[^"]*\(".*}\)$/\1=0.12.3\2/' test-fuzz/Cargo.toml

      - name: Test
        run: |
          OUTPUT="$(cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run 2>&1 1>/dev/null || true)"
          echo "$OUTPUT"
          echo "$OUTPUT" | grep '^`[^`]*` depends on `afl [^`]*`, which is newer than `cargo-afl [^`]*`.'
          echo "$OUTPUT" | grep 'Consider upgrading with `cargo install afl --force --version [^`]*`.$'
*/
#[test]
fn newer_afl() -> Result<()> {
    run_test(
        Some("0.12.2"),
        &[(
            r#"s/^\(afl = {.*\<version = "\)[^"]*\(".*}\)$/\1=0.12.3\2/"#,
            &["test-fuzz/Cargo.toml"],
        )],
        &[
            "^`[^`]*` depends on `afl [^`]*`, which is newer than `cargo-afl [^`]*`.",
            "Consider upgrading with `cargo install afl --force --version [^`]*`.$",
        ],
    )
}

/*
  test-incompatible-test-fuzz:
    ...
      - name: Install afl
        run: cargo install afl

      - name: Downgrade test-fuzz version
        run: |
          sed -i 's/^\(version = "\)[^.]*\.[^.]*\.\([^"]*"\)$/\10.0.\2/' test-fuzz/Cargo.toml
          sed -i 's/^\(test-fuzz = {.*\<version = "=\)[^.]*\.[^.]*\.\([^"]*".*}\)$/\10.0.\2/' cargo-test-fuzz/Cargo.toml examples/Cargo.toml

      - name: Test
        run: |
          OUTPUT="$(cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run 2>&1 1>/dev/null || true)"
          echo "$OUTPUT"
          echo "$OUTPUT" | grep '^Error: `[^`]*` depends on `test-fuzz [^`]*`, which is incompatible with `cargo-test-fuzz [^`]*`.$'
*/
#[test]
fn incompatible_test_fuzz() -> Result<()> {
    run_test(
        Some("*"),
        &[
            (
                r#"s/^\(version = "\)[^.]*\.[^.]*\.\([^"]*"\)$/\10.0.\2/"#,
                &["test-fuzz/Cargo.toml"],
            ),
            (
                r#"s/^\(test-fuzz = {.*\<version = "=\)[^.]*\.[^.]*\.\([^"]*".*}\)$/\10.0.\2/"#,
                &["cargo-test-fuzz/Cargo.toml", "examples/Cargo.toml"],
            ),
        ],
        &[
            "^Error: `[^`]*` depends on `test-fuzz [^`]*`, which is incompatible with \
             `cargo-test-fuzz [^`]*`.$",
        ],
    )
}

/*
  test-newer-test-fuzz:
    ...
      - name: Install afl
        run: cargo install afl

      - name: Upgrade test-fuzz version
        run: |
          sed -i 's/^\(version = "[^.]*\.[^.]*\)\.[^"]*\("\)$/\1.255\2/' test-fuzz/Cargo.toml
          sed -i 's/^\(test-fuzz = {.*\<version = "=[^.]*\.[^.]*\)\.[^"]*\(".*}\)$/\1.255\2/' cargo-test-fuzz/Cargo.toml examples/Cargo.toml
          sed -i 's/^\(version = "[^-]*\)-[^"]*\("\)$/\1\2/' cargo-test-fuzz/Cargo.toml

      - name: Test
        run: |
          OUTPUT="$(cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run 2>&1 1>/dev/null || true)"
          echo "$OUTPUT"
          echo "$OUTPUT" | grep '^`[^`]*` depends on `test-fuzz [^`]*`, which is newer than `cargo-test-fuzz [^`]*`.'
          echo "$OUTPUT" | grep 'Consider upgrading with `cargo install cargo-test-fuzz --force --version [^`]*`.$'
*/
#[test]
fn newer_test_fuzz() -> Result<()> {
    run_test(
        Some("*"),
        &[
            (
                r#"s/^\(version = "[^.]*\.[^.]*\)\.[^"]*\("\)$/\1.255\2/"#,
                &["test-fuzz/Cargo.toml"],
            ),
            (
                r#"s/^\(test-fuzz = {.*\<version = "=[^.]*\.[^.]*\)\.[^"]*\(".*}\)$/\1.255\2/"#,
                &["cargo-test-fuzz/Cargo.toml", "examples/Cargo.toml"],
            ),
            (
                r#"s/^\(version = "[^-]*\)-[^"]*\("\)$/\1\2/"#,
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
            cmd!(sh, "cargo install afl --version={version} --quiet").run()?;
        }

        for &(script, paths) in sed_scripts_and_paths {
            cmd!(sh, "sed -i {script} {paths...}").run()?;
        }

        let output = cmd!(
            sh,
            "cargo run -p cargo-test-fuzz -- test-fuzz -p test-fuzz-examples --no-run"
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

#[allow(unknown_lints, env_cargo_path)]
fn sandbox(f: impl FnOnce(Shell) -> Result<()>) -> Result<()> {
    let sh = Shell::new()?;

    let home = sh.create_temp_dir()?;
    sh.change_dir(home.path());

    sh.set_var("PATH", "/usr/bin");

    install_rust(&sh)?;

    // smoelius: `HOME` can be set only after Rust is installed. See, e.g.:
    // https://github.com/rust-lang/rustup/issues/1884#issuecomment-498157692
    sh.set_var("HOME", home.path());

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
