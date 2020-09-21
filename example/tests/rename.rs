use test_fuzz::test_fuzz;

#[test_fuzz(rename = "bar")]
pub fn foo() {}

// smoelius: Uncommenting the next line should produce a name collision.
// mod bar_fuzz {}

#[test]
fn test() {
    foo();
}
