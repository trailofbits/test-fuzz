use super::{Executable, TestFuzz};
use anyhow::{ensure, Result};
use internal::Fuzzer;
use semver::Version;
use std::{path::Path, process::Command};

mod impls;

pub(super) fn instantiate(fuzzer: Fuzzer) -> Result<&'static dyn Interface> {
    match fuzzer {
        Fuzzer::Aflplusplus => Ok(impls::aflplusplus::instantiate(false)),
        Fuzzer::AflplusplusPersistent => {
            // smoelius: https://github.com/AFLplusplus/AFLplusplus/blob/7b40d7b9420b2e3adb7d9afa88610199718dedba/include/forkserver.h#L114-L118
            ensure!(
                cfg!(not(target_os = "macos")),
                "`aflplusplus-persistent` is not supported on macOS"
            );
            Ok(impls::aflplusplus::instantiate(true))
        }
        Fuzzer::Libfuzzer => Ok(impls::libfuzzer::instantiate()),
    }
}

pub(super) trait Interface {
    fn to_enum(&self) -> Fuzzer;
    fn dependency_name(&self) -> Option<&'static str>;
    fn check_dependency_version(&self, executable_name: &str, version: &Version) -> Result<()>;
    fn build(&self, manifest_path: Option<&Path>) -> Result<Command>;
    fn fuzz(
        &self,
        opts: &TestFuzz,
        executable: &Executable,
        target: &str,
        input_dir: &Path,
        output_dir: &Path,
    ) -> Result<()>;
}
