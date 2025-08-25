use anyhow::Result;
use cargo_test_fuzz::{Object, TestFuzz, run};
use std::env;
use std::ffi::OsString;

mod transition;
use transition::cargo_test_fuzz;

pub fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<_> = env::args().map(OsString::from).collect();

    cargo_test_fuzz(&args)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    #[test]
    fn build_with_target() {
        #[allow(clippy::unwrap_used)]
        cargo_test_fuzz(&[
            "--features",
            &("test-fuzz/".to_owned() + test_fuzz::serde_format::as_feature()),
            "--no-run",
            "target",
        ])
        .unwrap();
    }

    fn cargo_test_fuzz(args: &[&str]) -> Result<()> {
        let mut cargo_test_fuzz_args = vec!["cargo-test-fuzz", "test-fuzz"];
        cargo_test_fuzz_args.extend_from_slice(args);
        super::cargo_test_fuzz(&cargo_test_fuzz_args)
    }
}
