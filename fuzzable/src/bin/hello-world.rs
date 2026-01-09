fn main() {
    intermediary("Hello, world!");
}

#[test_fuzz::test_fuzz(enable_in_production)]
fn intermediary(s: &str) {
    target(s);
}

fn target(s: &str) {
    println!("{s}");
}
