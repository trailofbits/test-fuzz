use internal::{dirs::corpus_directory_from_target, examples, testing::retry};
use predicates::prelude::*;
use std::fs::remove_dir_all;
use test_env_log::test;

const TIMEOUT: &str = "60";

#[test]
fn fuzz_assert() {
    fuzz("assert", false);
}

#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true);
}

fn fuzz(name: &str, persistent: bool) {
    let corpus = corpus_directory_from_target(name, &format!("{}::target", name));

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(name, &format!("{}::test", name))
        .unwrap()
        .assert()
        .success();

    retry(3, || {
        let mut command = examples::test_fuzz(name, &format!("{}::target", name)).unwrap();

        let mut args = vec!["--no-ui", "--run-until-crash"];
        if persistent {
            args.push("--persistent");
        }
        args.extend_from_slice(&["--", "-V", TIMEOUT]);

        command.args(&args).assert().success().try_stdout(
            predicate::str::contains("+++ Testing aborted programmatically +++")
                .and(predicate::str::contains("Time limit was reached").not()),
        )
    })
    .unwrap();
}
