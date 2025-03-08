use log::{error, info, warn};
use std::fmt::Display;

pub fn retry<O, T, E>(n: u64, operation: O) -> Result<T, E>
where
    O: Fn() -> Result<T, E>,
    T: Display,
    E: Display,
{
    assert!(n >= 1);

    for i in 1..=n {
        let result = operation();
        match &result {
            Ok(value) => {
                info!("{}", value);
                return result;
            }
            Err(error) if i < n => {
                warn!("{}", error);
            }
            Err(error) => {
                error!("{}", error);
                return result;
            }
        }
    }

    unreachable!();
}
