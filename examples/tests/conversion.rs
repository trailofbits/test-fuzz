use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PathBufWrapper(std::path::PathBuf);

impl From<&Path> for PathBufWrapper {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

impl test_fuzz::Into<&Path> for PathBufWrapper {
    fn into(self) -> &'static Path {
        Box::leak(Box::new(self.0))
    }
}

#[test_fuzz::test_fuzz(convert = "&Path, PathBufWrapper")]
fn target(path: &Path) {}

#[test]
fn test() {
    target(Path::new("/"));
}
