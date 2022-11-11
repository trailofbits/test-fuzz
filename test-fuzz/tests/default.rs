use internal::dirs::corpus_directory_from_target;
use std::fs::{read_dir, remove_dir_all};
use testing::examples;

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn no_default() {
    test("no_default", 0);
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn default() {
    test("default", 1);
}

fn test(name: &str, n: usize) {
    let corpus = corpus_directory_from_target("default", &format!("{name}::target"));

    // smoelius: `corpus` is distinct for all tests. So there is no race here.
    #[cfg_attr(
        dylint_lib = "non_thread_safe_call_in_test",
        allow(non_thread_safe_call_in_test)
    )]
    remove_dir_all(&corpus).unwrap_or_default();

    examples::test("default", &format!("{name}::target_fuzz::auto_generate"))
        .unwrap()
        .assert()
        .success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
