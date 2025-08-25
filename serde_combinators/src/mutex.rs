use super::{DeserializeWith, SerializeWith, compose_deserialize, compose_serialize};
use serde::{Deserializer, Serializer};
use std::{marker::PhantomData, sync::Mutex};

pub struct MutexF<W>(PhantomData<W>);

impl<W> SerializeWith for MutexF<W>
where
    W: SerializeWith,
{
    type T = Mutex<W::T>;

    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        compose_serialize(
            |value, serializer, serialize| {
                let value = Mutex::try_lock(value).map_err(serde::ser::Error::custom)?;
                serialize(&value, serializer)
            },
            W::serialize,
        )(value, serializer)
    }
}

impl<W> DeserializeWith for MutexF<W>
where
    W: DeserializeWith,
{
    type T = Mutex<W::T>;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>,
    {
        compose_deserialize(W::deserialize, |value| Ok(Mutex::new(value)))(deserializer)
    }
}
