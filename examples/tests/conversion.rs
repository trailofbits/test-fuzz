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

mod receiver {
    use serde::{Deserialize, Serialize};

    struct X;

    #[derive(Clone, Deserialize, Serialize)]
    struct Y;

    impl From<&X> for Y {
        fn from(_: &X) -> Self {
            Y
        }
    }

    impl test_fuzz::Into<&'static X> for Y {
        fn into(self) -> &'static X {
            Box::leak(Box::new(X))
        }
    }

    // smoelius: FIXME
    #[allow(clippy::use_self)]
    #[test_fuzz::test_fuzz_impl]
    impl X {
        #[test_fuzz::test_fuzz(convert = "&X, Y")]
        fn target(&self) {}
    }

    #[test]
    fn test() {
        X.target();
    }
}

#[cfg(feature = "__inapplicable_conversion")]
mod inapplicable_conversion {
    use serde::{Deserialize, Serialize};

    #[derive(Clone)]
    struct X;

    #[derive(Clone)]
    struct Y;

    #[derive(Clone, Deserialize, Serialize)]
    struct Z;

    impl From<Y> for Z {
        fn from(_: Y) -> Self {
            Z
        }
    }

    impl test_fuzz::Into<Y> for Z {
        fn into(self) -> Y {
            Y
        }
    }

    #[test_fuzz::test_fuzz_impl]
    impl X {
        #[test_fuzz::test_fuzz(convert = "Y, Z")]
        fn target(self) {}
    }

    #[test]
    fn test() {
        X.target();
    }
}
