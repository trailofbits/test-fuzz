mod command_ext;
pub use command_ext::CommandExt;

pub mod fuzzable;

mod retry;
pub use crate::retry::*;

#[ctor::ctor]
fn init() {
    internal::dirs::IN_TEST.store(true, std::sync::atomic::Ordering::SeqCst);

    env_logger::init();
}
