use super::{DeserializeWith, SerializeWith};
use serde::{de::DeserializeOwned, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

pub struct Type<T>(PhantomData<T>);

impl<T> SerializeWith for Type<T>
where
    T: Serialize,
{
    type T = T;

    fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::serialize(value, serializer)
    }
}

impl<T> DeserializeWith for Type<T>
where
    T: DeserializeOwned,
{
    type T = T;

    fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer)
    }
}
