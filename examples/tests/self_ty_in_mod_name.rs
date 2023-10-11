use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
struct Struct;

#[test_fuzz::test_fuzz_impl]
impl Struct {
    #[test_fuzz::test_fuzz]
    fn target(&self) {}
}

#[cfg(feature = "__self_ty_conflict")]
mod struct_target_fuzz {}
