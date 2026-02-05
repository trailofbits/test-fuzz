use assert_cmd::assert::Assert;
use log::debug;
use std::process::{Command, Output};

pub trait LoggedAssert {
    fn logged_assert(&mut self) -> Assert;
}

impl LoggedAssert for Command {
    fn logged_assert(&mut self) -> Assert {
        debug!("{self:?}");
        let output = self.output().unwrap();
        Assert::new(output_stripped_of_ansi_escapes(output))
    }
}

fn output_stripped_of_ansi_escapes(output: Output) -> Output {
    let Output {
        status,
        stdout,
        stderr,
    } = output;
    Output {
        status,
        stdout: strip_ansi_escapes::strip(stdout),
        stderr: strip_ansi_escapes::strip(stderr),
    }
}
