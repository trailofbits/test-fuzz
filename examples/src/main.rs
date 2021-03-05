fn main() {
    target("Hello, world!");
}

#[test_fuzz::test_fuzz(include_in_production)]
fn target(s: &str) {
    println!("{}", s);
}
