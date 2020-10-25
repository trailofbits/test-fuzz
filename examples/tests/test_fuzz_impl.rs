use serde::{Deserialize, Serialize};
use test_fuzz::{test_fuzz, test_fuzz_impl};

#[derive(Clone, Default, Deserialize, Serialize)]
struct Struct;

#[test_fuzz_impl]
impl Struct {
    #[test_fuzz]
    pub fn foo(&self) {}

    #[test_fuzz]
    pub fn bar(&mut self) {}
}

#[test]
fn foo() {
    Struct::default().foo();
}

#[test]
fn bar() {
    Struct::default().bar();
}
