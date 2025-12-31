use internal::dirs::corpus_directory_from_target;
use snapbox::{Redactions, assert_data_eq};
use std::{
    env::set_current_dir,
    fs::{read_to_string, remove_dir_all, remove_file},
    path::Path,
};
use testing::{CommandExt, examples};

#[test]
fn coverage() {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let expected_lcov_info = read_to_string(parent.join("etc/expected_lcov.info")).unwrap();

    set_current_dir(parent.join("examples")).unwrap();

    let corpus = corpus_directory_from_target("qwerty", "target");

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(corpus).unwrap_or_default();

    remove_file("lcov.info").unwrap_or_default();

    examples::test("qwerty", "test")
        .unwrap()
        .logged_assert()
        .success();

    dbg!(
        examples::test_fuzz("qwerty", "target")
            .unwrap()
            .arg("--coverage=corpus")
            .logged_assert()
            .success()
    );

    let contents = read_to_string("lcov.info").unwrap();

    let mut redactions = Redactions::new();
    redactions.insert("[REPO]", parent).unwrap();
    assert_data_eq!(redactions.redact(&contents), &expected_lcov_info);
}
