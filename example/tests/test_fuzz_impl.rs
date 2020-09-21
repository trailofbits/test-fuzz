use serde::{Deserialize, Serialize};
use test_fuzz::{test_fuzz, test_fuzz_impl};

#[derive(Clone, Default, Deserialize, Serialize)]
struct Struct;

#[test_fuzz_impl]
impl Struct {
    #[test_fuzz]
    pub fn foo(&self) {}

    #[allow(dead_code)]
    pub fn bar(&self) {}
}

#[test]
fn test() {
    Struct::default().foo();
}
