// smoelius: https://github.com/trailofbits/test-fuzz/issues/520
// Previously, `test_fuzz` would generate a module whose name was the target's name appended with
// `_fuzz`. If the target's name was `test`, a collision would result between `test_fuzz`'s package
// name and the generated module name. `test_fuzz` now avoids this problem by appending `_fuzz__`
// instead of `_fuzz`.
#[test_fuzz::test_fuzz]
fn test() {}
