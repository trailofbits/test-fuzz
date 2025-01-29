use serde::{Deserializer, Serializer};

/// Trait representing a serializing Serde combinator.
///
/// - [`With::T`] is the type to be serialized.
/// - [`With::serialize`] is what `#[serde(serialize_with = "...")]` expects.
///
/// See: <https://serde.rs/field-attrs.html#serialize_with>
pub trait SerializeWith {
    type T;
    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

/// Trait representing a deserializing Serde combinator.
///
/// - [`With::T`] is the type to be deserialized.
/// - [`With::deserialize`] is what `#[serde(deserialize_with = "...")]` expects.
///
/// See: <https://serde.rs/field-attrs.html#deserialize_with>
pub trait DeserializeWith {
    type T;
    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>;
}

/// Trait representing a Serde combinator.
///
/// The fields' roles are the same as in [`SerializeWith`] and [`DeserializeWith`].
pub trait With {
    type T;
    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>;
}

impl<W> With for W
where
    W: SerializeWith + DeserializeWith<T = <W as SerializeWith>::T>,
{
    type T = <W as SerializeWith>::T;
    fn serialize<S>(value: &Self::T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        W::serialize(value, serializer)
    }
    fn deserialize<'de, D>(deserializer: D) -> Result<Self::T, D::Error>
    where
        D: Deserializer<'de>,
    {
        W::deserialize(deserializer)
    }
}

/// Serializes a transformed value using `transform_and_serialize` and `serialize`, passing the
/// latter into the former.
///
/// Note that the most intuitive signature for this function would have an argument `transform`
/// rather than `transform_and_serialize`. However, some such `transform` functions would need to
/// return a reference to a local variable. An example occurs in [`RwLockF::serialize`]. Its
/// `transform` function would need to return a reference to a [`std::sync::RwLockReadGuard`].
fn compose_serialize<S, T, U, F>(
    transform_and_serialize: impl Fn(&T, S, F) -> Result<S::Ok, S::Error>,
    serialize: F,
) -> impl FnOnce(&T, S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    F: FnOnce(&U, S) -> Result<S::Ok, S::Error>,
{
    move |value, serializer| transform_and_serialize(value, serializer, serialize)
}

/// Deserializes using `deserialize` and then transforms the resulting value using `transform`.
fn compose_deserialize<'de, D, T, U>(
    deserialize: impl Fn(D) -> Result<U, D::Error>,
    transform: impl Fn(U) -> Result<T, D::Error>,
) -> impl Fn(D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer| {
        let value = deserialize(deserializer)?;
        transform(value)
    }
}

mod mutex;
mod ref_;
mod ref_mut;
mod rw_lock;
mod type_;

pub use mutex::MutexF;
pub use ref_::RefF;
pub use ref_mut::RefMutF;
pub use rw_lock::RwLockF;
pub use type_::Type;
