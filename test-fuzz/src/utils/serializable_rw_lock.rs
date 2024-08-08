use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Default)]
pub struct SerializableRwLock<T>(RwLock<T>);

impl<T> SerializableRwLock<T> {
    pub fn new(value: T) -> Self {
        Self(RwLock::<T>::new(value))
    }

    pub fn into_inner(self) -> LockResult<T> {
        self.0.into_inner()
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.0.read()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.0.write()
    }
}

impl<T> SerializableRwLock<T> {
    pub fn as_inner(&self) -> &RwLock<T> {
        &self.0
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for SerializableRwLock<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Self(RwLock::new(value)))
    }
}

impl<T: serde::Serialize> serde::Serialize for SerializableRwLock<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.read().unwrap().serialize(serializer)
    }
}
