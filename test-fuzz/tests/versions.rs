use cargo_metadata::{Dependency, DependencyKind, Metadata, MetadataCommand, Version};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

lazy_static! {
    static ref METADATA: Metadata = MetadataCommand::new().no_deps().exec().unwrap();
    static ref README_PATH: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("README.md");
}

#[test]
fn versions_are_equal() {
    for package in &METADATA.packages {
        assert_eq!(
            package.version.to_string(),
            env!("CARGO_PKG_VERSION"),
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

#[test]
#[ignore]
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
            .map(|match_| match_.map(|match_| match_.as_str()))
            .collect();
        assert_eq!(matches.last(), Some(&Some(env!("CARGO_PKG_VERSION"))));
    }
}
