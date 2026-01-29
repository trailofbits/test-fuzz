#[test_fuzz::test_fuzz(no_auto_generate = true)]
fn fail1(s: String) {
    let _ = s;
}

#[test_fuzz::test_fuzz(no_auto_generate = true)]
fn fail2(s: String) {
    let _ = s;
}
