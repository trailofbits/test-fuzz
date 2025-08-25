use super::{DeserializeWith, SerializeWith, compose_deserialize, compose_serialize};
use serde::{Deserializer, Serializer};
use std::{marker::PhantomData, sync::RwLock};

pub struct RwLockF<W>(PhantomData<W>);

impl<W> SerializeWith for RwLockF<W>
where
    W: SerializeWith,
{
    type T = RwLock<W::T>;

    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        compose_serialize(
            |value, serializer, serialize| {
                let value = RwLock::try_read(value).map_err(serde::ser::Error::custom)?;
                serialize(&value, serializer)
            },
            W::serialize,
        )(value, serializer)
    }
}

impl<W> DeserializeWith for RwLockF<W>
where
    W: DeserializeWith,
{
    type T = RwLock<W::T>;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>,
    {
        compose_deserialize(W::deserialize, |value| Ok(RwLock::new(value)))(deserializer)
    }
}
