use dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::remove_dir_all;

const TIMEOUT: &str = "60";

#[test]
fn fuzz_assert() {
    fuzz("assert", false)
}

#[test]
fn fuzz_qwerty() {
    fuzz("qwerty", true)
}

fn fuzz(name: &str, persistent: bool) {
    let corpus = corpus_directory_from_target(name, &format!("{}::target", name));

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(name, &format!("{}::test", name))
        .unwrap()
        .assert()
        .success();

    let mut command = examples::test_fuzz(&format!("{}::target", name)).unwrap();

    let mut args = vec!["--no-ui", "--run-until-crash"];
    if persistent {
        args.push("--persistent");
    }
    args.extend_from_slice(&["--", "-V", TIMEOUT]);

    command.args(&args).assert().success().stdout(
        predicate::str::contains("+++ Testing aborted programmatically +++")
            .and(predicate::str::contains("Time limit was reached").not()),
    );
}
