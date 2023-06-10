pub use internal;

mod command_ext;
pub use command_ext::CommandExt;

pub mod examples;

mod retry;
pub use crate::retry::*;

#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[macro_export]
macro_rules! skip_fuzzer {
    ($test:expr, $fuzzer:expr) => {
        if $crate::internal::fuzzer().unwrap() == $fuzzer {
            use std::io::Write;
            writeln!(
                std::io::stderr(),
                "Skipping `{}` test for `{}`",
                $test,
                $fuzzer
            )
            .unwrap();
            return;
        }
    };
}
