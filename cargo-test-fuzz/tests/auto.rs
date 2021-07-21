use dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};

const TIMEOUT: &str = "60";

// smoelius: It would be nice if these first two tests could distinguish how man "auto" tests get
// run (0 vs. 1). But right now, I can't think of an easy way to do this.

#[test]
fn no_auto() {
    auto(
        "alloc",
        "alloc::target",
        false,
        "Could not find or auto-generate",
        0,
    );
}

#[test]
fn auto_empty() {
    auto(
        "no_default",
        "no_default::target",
        false,
        "Could not find or auto-generate",
        0,
    );
}

#[test]
fn auto_nonempty() {
    auto("assert", "assert::target", true, "Auto-generated", 1);
}

fn auto(krate: &str, target: &str, success: bool, pattern: &str, n: usize) {
    let corpus = corpus_directory_from_target(krate, target);

    remove_dir_all(&corpus).unwrap_or_default();

    let assert = examples::test_fuzz(target)
        .unwrap()
        .args(&["--no-ui", "--run-until-crash", "--", "-V", TIMEOUT])
        .assert();

    let assert = if success {
        assert.success()
    } else {
        assert.failure()
    };

    assert.stderr(predicate::str::contains(pattern));

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
