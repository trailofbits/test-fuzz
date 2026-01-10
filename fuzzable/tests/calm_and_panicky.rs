//! The function [`calm`] intentionally returns immediately. The function [`panicky`] intentionally
//! panics.

// smoelius: FIXME
#![cfg_attr(dylint_lib = "general", allow(crate_wide_allow))]
#![allow(clippy::used_underscore_binding)]

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
#[should_panic = "explicit panic"]
fn test() {
    calm(Calm(false));
    panicky(Panicky(false));
}
