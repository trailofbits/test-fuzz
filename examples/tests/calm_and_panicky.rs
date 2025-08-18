//! The function [`calm`] intentionally returns immediately. The function [`panicky`] intentionally
//! panics.

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Calm(bool);

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Panicky(bool);

#[test_fuzz::test_fuzz]
fn calm(_calm: Calm) {}

#[test_fuzz::test_fuzz]
fn panicky(_panicky: Panicky) {
    panic!();
}

#[test]
#[should_panic]
fn test() {
    calm(Calm(false));
    panicky(Panicky(false));
}
