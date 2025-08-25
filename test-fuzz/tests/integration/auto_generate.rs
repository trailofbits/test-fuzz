use internal::dirs::corpus_directory_from_target;
use std::fs::{read_dir, remove_dir_all};
use testing::{CommandExt, examples};

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn signed() {
    test("signed", 6);
}

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn unsigned() {
    test("unsigned", 6);
}

fn test(name: &str, n: usize) {
    let corpus = corpus_directory_from_target("auto_generate", &format!("{name}::target"));

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(
        "auto_generate",
        &format!("{name}::target_fuzz__::auto_generate"),
    )
    .unwrap()
    .logged_assert()
    .success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
