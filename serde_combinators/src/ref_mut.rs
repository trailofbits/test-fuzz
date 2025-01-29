use super::{compose_deserialize, compose_serialize, DeserializeWith, SerializeWith};
use serde::{Deserializer, Serializer};
use std::marker::PhantomData;

pub struct RefMutF<'a, W>(PhantomData<&'a mut W>);

impl<'a, W> SerializeWith for RefMutF<'a, W>
where
    W: SerializeWith,
{
    type T = &'a mut W::T;

    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        compose_serialize(
            |value, serializer, serialize| serialize(value, serializer),
            W::serialize,
        )(value, serializer)
    }
}

impl<'a, W> DeserializeWith for RefMutF<'a, W>
where
    W: DeserializeWith,
{
    type T = &'a mut W::T;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>,
    {
        compose_deserialize(W::deserialize, |value| {
            Ok(Box::leak(Box::new(value)) as &mut _)
        })(deserializer)
    }
}
