use internal::dirs::corpus_directory_from_target;
use std::{
    fs::{read_dir, remove_dir_all, File},
    io::Read,
};
use testing::examples;

#[test]
fn serde_format() {
    let corpus = corpus_directory_from_target("serde", "unit_variant::target");

    remove_dir_all(&corpus).unwrap_or_default();

    examples::test("serde", "unit_variant::test")
        .unwrap()
        .assert()
        .success();

    for entry in read_dir(corpus).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        // smoelius: CBOR stores the variant name.
        assert_eq!(
            cfg!(any(feature = "serde_cbor", feature = "serde_cbor4ii")),
            buf.iter().any(u8::is_ascii_uppercase)
        );
    }
}
