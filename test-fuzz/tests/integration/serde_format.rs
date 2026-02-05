use internal::dirs::corpus_directory_from_target;
use std::{
    fs::{File, read_dir, remove_dir_all},
    io::Read,
};
use testing::{LoggedAssert, fuzzable};

#[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
#[test]
fn serde_format() {
    let corpus = corpus_directory_from_target("serde", "unit_variant::target");

    remove_dir_all(&corpus).unwrap_or_default();

    fuzzable::test("serde", "unit_variant::test")
        .unwrap()
        .logged_assert()
        .success();

    for entry in read_dir(corpus).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        // smoelius: CBOR stores the variant name. Hence, this test will fail if CBOR is used as the
        // serialization format.
        // smoelius: CBOR is no longer a supported format. Still, I see no reason to remove this
        // test or the next check.
        assert!(!buf.iter().any(u8::is_ascii_uppercase));
    }
}
