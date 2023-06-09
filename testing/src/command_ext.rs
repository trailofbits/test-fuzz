use assert_cmd::{assert::Assert, Command};
use log::debug;

pub trait CommandExt {
    fn logged_assert(&mut self) -> Assert;
}

impl CommandExt for Command {
    fn logged_assert(&mut self) -> Assert {
        debug!("{:?}", self);
        #[allow(clippy::disallowed_methods)]
        Self::assert(self)
    }
}
