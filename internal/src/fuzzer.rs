use anyhow::{anyhow, Result};
use clap::ValueEnum;
use heck::ToSnakeCase;
use serde::{Deserialize, Serialize};
use std::env::var;
use strum_macros::{Display, EnumIter};

// smoelius: The user selects the fuzzer via an environment variable (`TEST_FUZZ_FUZZER`) or a
// command line option. Hence, cargo-test-fuzz needs to know all available fuzzers. This is unlike
// the Serde format, which the user selects via a Cargo feature.
#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumIter, PartialEq, Eq, Serialize, ValueEnum,
)]
#[remain::sorted]
pub enum Fuzzer {
    Aflplusplus,
    AflplusplusPersistent,
    Libfuzzer,
}

const DEFAULT_FUZZER: Fuzzer = Fuzzer::Aflplusplus;

pub fn fuzzer() -> Result<Fuzzer> {
    match var("TEST_FUZZ_FUZZER") {
        Ok(value) => {
            let fuzzer = Fuzzer::from_str(&value, false).map_err(|error| {
                anyhow!(
                    "`TEST_FUZZ_FUZZER` is set to {value:?}, which could not be parsed: {error}"
                )
            })?;
            Ok(fuzzer)
        }
        Err(_) => Ok(DEFAULT_FUZZER),
    }
}

impl Fuzzer {
    #[must_use]
    pub fn as_feature(self) -> String {
        String::from("__fuzzer_") + &self.to_string().to_snake_case()
    }
}
