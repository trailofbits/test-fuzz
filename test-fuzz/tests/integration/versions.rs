use cargo_metadata::{Dependency, DependencyKind, Metadata, MetadataCommand};
use once_cell::sync::Lazy;
use semver::Version;

static METADATA: Lazy<Metadata> = Lazy::new(|| MetadataCommand::new().no_deps().exec().unwrap());

#[test]
fn versions_are_equal() {
    for package in &METADATA.packages {
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            package.version.to_string(),
            "{}",
            package.name
        );
    }
}

#[test]
fn versions_are_exact_and_match() {
    for package in &METADATA.packages {
        for Dependency {
            name: dep,
            req,
            kind,
            ..
        } in &package.dependencies
        {
            if dep.starts_with("test-fuzz") && kind != &DependencyKind::Development {
                assert!(
                    req.to_string().starts_with('='),
                    "`{}` dependency on `{}` is not exact",
                    package.name,
                    dep
                );
                assert!(
                    req.matches(&Version::parse(env!("CARGO_PKG_VERSION")).unwrap()),
                    "`{}` dependency on `{}` does not match `{}`",
                    package.name,
                    dep,
                    env!("CARGO_PKG_VERSION"),
                );
            }
        }
    }
}

// #[test]
#[allow(dead_code)]
fn afl_version_is_exact() {
    for package in &METADATA.packages {
        for Dependency { name: dep, req, .. } in &package.dependencies {
            if dep == "afl" {
                assert!(
                    req.to_string().starts_with('='),
                    "`{}` dependency on `{}` is not exact",
                    package.name,
                    dep
                );
            }
        }
    }
}
