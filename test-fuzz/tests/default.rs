use internal::{dirs::corpus_directory_from_target, examples};
use std::fs::{read_dir, remove_dir_all};

#[test]
fn no_default() {
    test("no_default", 0);
}

#[test]
fn default() {
    test("default", 1);
}

fn test(name: &str, n: usize) {
    let corpus = corpus_directory_from_target("default", &format!("{}::target", name));

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test("default", &format!("{}::target_fuzz::auto_generate", name))
        .unwrap()
        .assert()
        .success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
