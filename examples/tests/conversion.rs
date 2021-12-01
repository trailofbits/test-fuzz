mod path {
    use std::path::Path;
    use test_fuzz::leak;

    leak!(Path, LeakedPath);

    #[test_fuzz::test_fuzz(convert = "&Path, LeakedPath")]
    fn target(path: &Path) {}

    #[test]
    fn test() {
        target(Path::new("/"));
    }
}

mod array {
    use test_fuzz::leak;

    leak!([String], LeakedArray);

    #[test_fuzz::test_fuzz(convert = "&[String], LeakedArray")]
    fn target(path: &[String]) {}

    #[test]
    fn test() {
        target(&[String::from("x")]);
    }
}
