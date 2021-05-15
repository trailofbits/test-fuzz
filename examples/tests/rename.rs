#[test_fuzz::test_fuzz(rename = "bar")]
fn foo() {}

// smoelius: Building with feature `bar_fuzz` should produce a name collision.
#[cfg(feature = "bar_fuzz")]
mod bar_fuzz {}

#[test]
fn test() {
    foo();
}
