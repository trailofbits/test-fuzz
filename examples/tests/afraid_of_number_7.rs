// smoelius: This target is based on one from `cargo-fuzz`:
// https://github.com/rust-fuzz/cargo-fuzz/blob/f52846dd963948ed3728fde3aa759195222c5933/tests/tests/project.rs#L152-L156
// It is included here to help verify the correct integration of `libfuzzer`.

#[allow(clippy::manual_assert)]
pub fn fail_fuzzing(data: &[u8]) {
    if data.len() == 7 {
        panic!("I'm afraid of number 7");
    }
}

#[test]
fn test() {
    fail_fuzzing(&[]);
}
