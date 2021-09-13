/// Skip values of type `$ty` when serializing. Initialize values of type `$ty` with `$expr` when
/// deserializing.
///
/// **Warning:** `dont_care!` is provided for convenience and may be removed in future versions of
/// `test-fuzz`.
#[macro_export]
macro_rules! dont_care {
    ($ty:path, $expr:expr) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                ().serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
