mod command_ext;
use std::sync::atomic::{AtomicU32, Ordering};

pub use command_ext::CommandExt;

pub mod examples;

mod retry;
pub use crate::retry::*;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

static ID: AtomicU32 = AtomicU32::new(0);

pub fn unique_id() -> String {
    ID.fetch_add(1, Ordering::SeqCst).to_string()
}
