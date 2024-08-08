use std::sync::{LockResult, Mutex, MutexGuard};

#[derive(Debug, Default)]
pub struct SerializableMutex<T>(Mutex<T>);

impl<T> SerializableMutex<T> {
    pub fn new(value: T) -> Self {
        Self(Mutex::<T>::new(value))
    }

    pub fn into_inner(self) -> LockResult<T> {
        self.0.into_inner()
    }

    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        self.0.lock()
    }
}

impl<T> SerializableMutex<T> {
    pub fn as_inner(&self) -> &Mutex<T> {
        &self.0
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for SerializableMutex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Self(Mutex::new(value)))
    }
}

impl<T: serde::Serialize> serde::Serialize for SerializableMutex<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.lock().unwrap().serialize(serializer)
    }
}
