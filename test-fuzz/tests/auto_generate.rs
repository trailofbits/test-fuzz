use internal::{dirs::corpus_directory_from_target, examples};
use std::fs::{read_dir, remove_dir_all};

#[test]
fn signed() {
    test("signed", 6);
}

#[test]
fn unsigned() {
    test("unsigned", 6);
}

fn test(name: &str, n: usize) {
    let corpus = corpus_directory_from_target("auto_generate", &format!("{}::target", name));

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test(
        "auto_generate",
        &format!("{}::target_fuzz::auto_generate", name),
    )
    .unwrap()
    .assert()
    .success();

    assert_eq!(read_dir(corpus).map(Iterator::count).unwrap_or_default(), n);
}
