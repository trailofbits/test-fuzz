use cargo_metadata::{Dependency, Metadata, MetadataCommand};
use lazy_static::lazy_static;
use semver::Version;

lazy_static! {
    static ref METADATA: Metadata = MetadataCommand::new().no_deps().exec().unwrap();
    static ref CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

#[test]
fn check_versions_are_equal() {
    for package in &METADATA.packages {
        assert_eq!(
            package.version.to_string(),
            *CRATE_VERSION,
            "{}",
            package.name
        );
    }
}

#[test]
fn check_versions_are_exact_and_match() {
    for package in &METADATA.packages {
        for Dependency { name: dep, req, .. } in &package.dependencies {
            if dep.starts_with("test-fuzz") {
                assert!(
                    req.is_exact(),
                    "`{}` dependency on `{}` is not exact",
                    package.name,
                    dep
                );
                assert!(
                    req.matches(&Version::parse(*CRATE_VERSION).unwrap()),
                    "`{}` dependency on `{}` does not match `{}`",
                    package.name,
                    dep,
                    *CRATE_VERSION,
                );
            }
        }
    }
}
