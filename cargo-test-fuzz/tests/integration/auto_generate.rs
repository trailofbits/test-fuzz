use anyhow::ensure;
use internal::dirs::corpus_directory_from_target;
use predicates::prelude::*;
use std::fs::{read_dir, remove_dir_all};
use testing::{LoggedAssert, fuzzable, retry};

const MAX_TOTAL_TIME: &str = "60";

// smoelius: It would be nice if these first two tests could distinguish how many "auto_generate"
// tests get run (0 vs. 1). But right now, I can't think of an easy way to do this.

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn no_auto_generate() {
    auto_generate(
        "alloc",
        "target",
        false,
        "Could not find or auto-generate",
        0,
    );
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn auto_generate_empty() {
    auto_generate("default", "no_default::target", false, "", 0);
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn auto_generate_nonempty() {
    auto_generate("assert", "target", true, "Auto-generated", 1);
}

fn auto_generate(krate: &str, target: &str, success: bool, pattern: &str, n: usize) {
    let corpus = corpus_directory_from_target(krate, target);

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(&corpus).unwrap_or_default();

    let _: &str = retry(3, || {
        let assert = fuzzable::test_fuzz(krate, target)
            .unwrap()
            .args([
                "--no-ui",
                "--run-until-crash",
                "--max-total-time",
                MAX_TOTAL_TIME,
            ])
            .logged_assert();

        let assert = if success {
            assert.success()
        } else {
            assert.failure()
        };

        assert.try_stderr(predicate::str::contains(pattern))?;

        ensure!(read_dir(&corpus).map(Iterator::count).unwrap_or_default() == n);

        Ok("")
    })
    .unwrap();
}
