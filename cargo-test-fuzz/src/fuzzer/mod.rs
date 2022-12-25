use super::{Executable, TestFuzz};
use anyhow::Result;
use internal::Fuzzer;
use semver::Version;
use std::{path::Path, process::Command};

mod impls;

pub(super) fn instantiate(fuzzer: Fuzzer) -> &'static dyn Interface {
    match fuzzer {
        Fuzzer::Aflplusplus => impls::aflplusplus::instantiate(false),
        Fuzzer::AflplusplusPersistent => impls::aflplusplus::instantiate(true),
        Fuzzer::Libfuzzer => impls::libfuzzer::instantiate(),
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
