fn main() {
    target("Hello, world!");
}

#[test_fuzz::test_fuzz(enable_in_production)]
fn target(s: &str) {
    println!("{}", s);
}
