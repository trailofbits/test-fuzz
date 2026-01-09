// smoelius: `VarGuard` is based on:
// https://github.com/rust-lang/rust-clippy/blob/9cc8da222b3893bc13bc13c8827e93f8ea246854/tests/compile-test.rs
// smoelius: Clippy dropped `VarGuard` when it switched to `ui_test`:
// https://github.com/rust-lang/rust-clippy/commit/77d10ac63dae6ef0a691d9acd63d65de9b9bf88e

use std::{
    env::{remove_var, set_var, var_os},
    ffi::{OsStr, OsString},
};

/// Restores an env var on drop
#[derive(Debug)]
#[must_use]
pub struct VarGuard {
    key: &'static str,
    value: Option<OsString>,
}

impl VarGuard {
    pub fn set(key: &'static str, val: impl AsRef<OsStr>) -> Self {
        let value = var_os(key);
        unsafe {
            set_var(key, val);
        }
        Self { key, value }
    }
}

impl Drop for VarGuard {
    fn drop(&mut self) {
        match self.value.as_deref() {
            None => unsafe { remove_var(self.key) },
            Some(value) => unsafe { set_var(self.key, value) },
        }
    }
}
