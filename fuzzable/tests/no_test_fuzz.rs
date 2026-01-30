#[allow(dead_code)]
struct Struct;

#[cfg_attr(feature = "__no_test_fuzz", test_fuzz::test_fuzz_impl)]
impl Struct {}
