mod command_ext;
pub use command_ext::CommandExt;

pub mod examples;

mod retry;
pub use crate::retry::*;

#[ctor::ctor]
fn init() {
    env_logger::init();
}
