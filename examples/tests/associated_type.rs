// smoelius: Associated types are considered a legitimate reason to put a bound on a struct
// parameter. See:
// * https://github.com/rust-lang/rust-clippy/issues/1689
// * https://github.com/rust-lang/api-guidelines/issues/6
//
// This example is based in part on:
// https://docs.serde.rs/serde_json/#creating-json-by-serializing-data-structures

mod associated_type {
    use serde::{de, Deserialize, Serialize};

    trait Serializable {
        type Out: Clone + de::DeserializeOwned + Serialize + PartialEq + Eq;
        fn serialize(&self) -> Self::Out;
    }

    impl<T> Serializable for T
    where
        T: Serialize,
    {
        type Out = String;
        fn serialize(&self) -> Self::Out {
            serde_json::to_string(self).unwrap()
        }
    }

    #[test_fuzz::test_fuzz(concretize = "Address", bounds = "T: Serializable")]
    fn serializes_to<T>(x: &T, y: &T::Out) -> bool
    where
        T: Clone + de::DeserializeOwned + Serialize + Serializable,
    {
        &<T as Serializable>::serialize(x) == y
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct Address {
        street: String,
        city: String,
    }

    #[test]
    fn test() {
        let address = Address {
            street: "10 Downing Street".to_owned(),
            city: "London".to_owned(),
        };

        assert!(serializes_to(
            &address,
            &String::from("{\"street\":\"10 Downing Street\",\"city\":\"London\"}")
        ));
    }
}
