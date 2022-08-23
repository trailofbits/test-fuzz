pub trait FromRef<T> {
    fn from_ref(_: &T) -> Self;
}

impl<T, U> FromRef<T> for U
where
    T: Clone,
    U: From<T>,
{
    fn from_ref(value: &T) -> Self {
        Self::from(value.clone())
    }
}

pub trait Into<T> {
    fn into(self) -> T;
}
