use super::{DeserializeWith, SerializeWith, compose_deserialize, compose_serialize};
use serde::{Deserializer, Serializer};
use std::marker::PhantomData;

pub struct RefF<'a, W>(PhantomData<&'a W>);

impl<'a, W> SerializeWith for RefF<'a, W>
where
    W: SerializeWith,
{
    type T = &'a W::T;

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

impl<'a, W> DeserializeWith for RefF<'a, W>
where
    W: DeserializeWith,
{
    type T = &'a W::T;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>,
    {
        compose_deserialize(W::deserialize, |value| Ok(Box::leak(Box::new(value)) as &_))(
            deserializer,
        )
    }
}
