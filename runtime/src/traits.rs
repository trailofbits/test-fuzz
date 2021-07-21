use core::ops::{Add, Div, Sub};
use num_traits::{Bounded, One};

pub trait MinValueAddOne {
    fn min_value_add_one() -> Self;
}

impl<T> MinValueAddOne for T
where
    T: Add<Output = Self> + Bounded + One,
{
    fn min_value_add_one() -> Self {
        Self::min_value() + Self::one()
    }
}

pub trait MaxValueSubOne {
    fn max_value_sub_one() -> Self;
}

impl<T> MaxValueSubOne for T
where
    T: Sub<Output = Self> + Bounded + One,
{
    fn max_value_sub_one() -> Self {
        Self::max_value() - Self::one()
    }
}

pub trait Two {
    fn two() -> Self;
}

impl<T> Two for T
where
    T: Add<Output = Self> + One,
{
    fn two() -> Self {
        Self::one() + Self::one()
    }
}

pub trait Middle {
    fn low() -> Self;
    fn high() -> Self;
}

impl<T> Middle for T
where
    T: Add<Output = Self> + Div<Output = Self> + Bounded + One + Two,
{
    fn low() -> Self {
        Self::min_value() / Self::two() + Self::max_value() / Self::two()
    }
    fn high() -> Self {
        Self::low() + Self::one()
    }
}
