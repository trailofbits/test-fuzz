#![allow(deprecated)]

use anyhow::Result;
use cargo_test_fuzz::cargo_test_fuzz;
use std::env;
use std::ffi::OsString;

pub fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<_> = env::args().map(OsString::from).collect();

    cargo_test_fuzz(&args)
}
