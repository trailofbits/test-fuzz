#![cfg_attr(dylint_lib = "general", allow(crate_wide_allow))]
#![allow(unused)]

use std::sync::Mutex;

// Traits `serde::Serialize` and `serde::Deserialize` cannot be derived for `Context` because it
// contains a `Mutex`.
#[derive(Default)]
struct Context {
    lock: Mutex<()>,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
}

#[test_fuzz::test_fuzz]
fn target(#[serde(skip)] context: Context, x: i32) {
    assert!(x >= 0);
}

#[test]
fn test() {
    target(Context::default(), 0);
}
