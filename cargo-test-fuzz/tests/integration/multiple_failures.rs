use predicates::prelude::*;
use std::fs::remove_dir_all;
use testing::{CommandExt, fuzzable};

#[test]
fn multiple_failures() {
    let krate = "multiple_failures";

    let targets = ["fail1", "fail2"];
    for target in targets {
        let corpus = internal::dirs::corpus_directory_from_target(krate, target);
        remove_dir_all(&corpus).unwrap_or_default();
    }

    fuzzable::test_fuzz_all()
        .unwrap()
        .args([
            "--test", krate,
            "--no-ui",
            "--run-until-crash",
            "--max-total-time", "1",
        ])
        .logged_assert()
        .failure()
        .stderr(predicate::str::is_match(r"(?s)Could not find or auto-generate `.*multiple_failures::fail1`\. Please ensure `fail1` is tested\..*Could not find or auto-generate `.*multiple_failures::fail2`\. Please ensure `fail2` is tested\.").unwrap());
}
