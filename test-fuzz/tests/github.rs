use heck::ToKebabCase;
use internal::Fuzzer;
use itertools::Itertools;
use std::{fs::read_to_string, path::Path};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yaml_rust::YamlLoader;

// smoelius: We cannot use `internal::SerdeFormat` because only one of its variants will be enabled.
#[derive(Clone, Copy, Debug, Display, EnumIter)]
enum SerdeFormat {
    Bincode,
    Cbor,
    Cbor4ii,
}

#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, PartialEq)]
enum Environment {
    UbuntuLatest,
    MacosLatest,
}

#[derive(Clone, Copy, Debug, Display, EnumIter)]
enum Toolchain {
    Stable,
    Nightly,
}

#[test]
fn matrix() {
    let expecteds = std::iter::repeat(
        Fuzzer::iter()
            .cartesian_product(Environment::iter())
            .filter(|&(fuzzer, environment)| {
                fuzzer != Fuzzer::AflplusplusPersistent || environment != Environment::MacosLatest
            }),
    )
    .take(2)
    .flatten()
    .zip(
        SerdeFormat::iter()
            .cartesian_product(Toolchain::iter())
            .cycle(),
    );

    let ci_yml = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/ci.yml");
    let contents = read_to_string(ci_yml).unwrap();
    let doc = YamlLoader::load_from_str(&contents)
        .ok()
        .and_then(|vec| vec.into_iter().next())
        .unwrap();
    let actuals = doc["jobs"]["test"]["strategy"]["matrix"]["include"]
        .as_vec()
        .unwrap();

    assert_eq!(expecteds.clone().count(), actuals.len());

    for (expected, actual) in expecteds.zip(actuals) {
        let ((expected_fuzzer, expected_environment), (expected_serde_format, expected_toolchain)) =
            expected;
        if expected_fuzzer == Fuzzer::AflplusplusPersistent
            && expected_environment == Environment::MacosLatest
        {
            continue;
        }
        assert_eq!(
            expected_fuzzer.to_string().to_kebab_case(),
            actual["fuzzer"].as_str().unwrap()
        );
        assert_eq!(
            expected_serde_format.to_string().to_kebab_case(),
            actual["serde_format"].as_str().unwrap()
        );
        assert_eq!(
            expected_environment.to_string().to_kebab_case(),
            actual["environment"].as_str().unwrap()
        );
        assert_eq!(
            expected_toolchain.to_string().to_kebab_case(),
            actual["toolchain"].as_str().unwrap()
        );
    }
}
