//! **Warning:** The macros in `test_fuzz::utils` are provided for convenience and may be removed in
//! future versions of `test-fuzz`.

/// Skip values of type `$ty` when serializing. Initialize values of type `$ty` with `$expr` when
/// deserializing.
#[macro_export]
macro_rules! dont_care {
    ($ty:path, $expr:expr) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                ().serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <()>::deserialize(deserializer).map(|_| $expr)
            }
        }
    };
    ($ty:path) => {
        $crate::dont_care!($ty, $ty);
    };
}

/// Wrap `<$ty as ToOwned>::Owned` in a type `$ident` and implement `From` and `test_fuzz::Into`
/// for `$ident` so that `convert = "&$ty, $ident"` can be used.
#[macro_export]
macro_rules! leak {
    ($ty:ty, $ident:ident) => {
        #[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
        struct $ident(<$ty as ToOwned>::Owned);

        impl From<&$ty> for $ident {
            fn from(ty: &$ty) -> Self {
                Self(ty.to_owned())
            }
        }

        impl test_fuzz::Into<&$ty> for $ident {
            fn into(self) -> &'static $ty {
                Box::leak(Box::new(self.0))
            }
        }
    };
    ([$ty:ty], $ident:ident) => {
        #[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
        struct $ident(<[$ty] as ToOwned>::Owned);

        impl From<&[$ty]> for $ident {
            fn from(ty: &[$ty]) -> Self {
                Self(ty.to_owned())
            }
        }

        impl test_fuzz::Into<&[$ty]> for $ident {
            fn into(self) -> &'static [$ty] {
                Box::leak(Box::new(self.0))
            }
        }
    };
}
