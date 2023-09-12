use cargo_metadata::{Dependency, DependencyKind, Metadata, MetadataCommand};
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use semver::Version;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

static METADATA: Lazy<Metadata> = Lazy::new(|| MetadataCommand::new().no_deps().exec().unwrap());

#[allow(unknown_lints, env_cargo_path)]
static README_PATH: Lazy<PathBuf> =
    Lazy::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../README.md"));

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

#[test]
fn readme_references_current_version() {
    let re = Regex::new(r#"(test-fuzz|version) = "([^"]*)""#).unwrap();
    let content = read_to_string(&*README_PATH).unwrap();
    for group in re.captures_iter(&content) {
        let matches: Vec<Option<&str>> = group
            .iter()
            .map(|match_| match_.as_ref().map(Match::as_str))
            .collect();
        let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        assert_eq!(
            matches.last(),
            Some(&Some(
                format!("{}.{}", version.major, version.minor).as_str()
            ))
        );
    }
}
