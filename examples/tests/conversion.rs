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

    #[derive(Clone)]
    struct X;

    #[derive(Clone, Deserialize, Serialize)]
    struct Y;

    impl From<&X> for Y {
        fn from(_: &X) -> Self {
            Self
        }
    }

    impl From<Y> for &X {
        fn from(_: Y) -> Self {
            Box::leak(Box::new(X))
        }
    }

    // smoelius: FIXME
    #[allow(clippy::use_self)]
    #[test_fuzz::test_fuzz_impl]
    impl X {
        #[test_fuzz::test_fuzz(convert = "&X, Y")]
        fn target_self(&self) {}

        #[test_fuzz::test_fuzz(convert = "&X, Y")]
        fn target_other(other: &X) {}
    }

    #[test]
    fn test() {
        X.target_self();
        X::target_other(&X);
    }
}

mod lifetime {
    use serde::{Deserialize, Serialize};

    #[derive(Clone)]
    struct X<'a>(&'a bool);

    #[derive(Clone, Deserialize, Serialize)]
    struct Y(bool);

    impl<'a> From<X<'a>> for Y {
        fn from(x: X) -> Self {
            Self(*x.0)
        }
    }

    impl<'a> From<Y> for X<'a> {
        fn from(value: Y) -> Self {
            Self(Box::leak(Box::new(value.0)))
        }
    }

    #[test_fuzz::test_fuzz(convert = "X<'a>, Y")]
    fn target<'a>(x: X<'a>) {}

    #[test]
    fn test() {
        target(X(&false));
    }
}

mod mutable {
    use serde::{Deserialize, Serialize};

    #[derive(Clone)]
    struct X(bool);

    #[derive(Clone, Deserialize, Serialize)]
    struct Y(bool);

    impl From<X> for Y {
        fn from(x: X) -> Self {
            Self(x.0)
        }
    }

    impl From<Y> for X {
        fn from(value: Y) -> Self {
            Self(value.0)
        }
    }

    #[test_fuzz::test_fuzz(convert = "X, Y")]
    fn target(mut x: X) {
        x.0 = true;
    }

    #[test]
    fn test() {
        target(X(false));
    }
}

mod uncloneable {
    use serde::{Deserialize, Serialize};

    struct X;

    #[derive(Clone, Deserialize, Serialize)]
    struct Y;

    impl test_fuzz::FromRef<X> for Y {
        fn from_ref(_: &X) -> Self {
            Self
        }
    }

    impl From<Y> for X {
        fn from(_: Y) -> Self {
            Self
        }
    }

    #[test_fuzz::test_fuzz(convert = "X, Y")]
    fn target(x: X) {}

    #[test]
    fn test() {
        target(X);
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
            Self
        }
    }

    impl From<Z> for Y {
        fn from(_: Z) -> Self {
            Self
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
