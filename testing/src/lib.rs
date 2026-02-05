mod logged_assert;
pub use logged_assert::LoggedAssert;

pub mod fuzzable;

mod retry;
pub use crate::retry::*;

#[ctor::ctor]
fn init() {
    internal::dirs::IN_TEST.store(true, std::sync::atomic::Ordering::SeqCst);

    env_logger::init();
}
