use log::{error, info, warn};
use retry::delay::Fixed;
use std::fmt::Debug;

pub fn retry<O, T, E>(n: u64, operation: O) -> Result<T, retry::Error<E>>
where
    O: Fn() -> std::result::Result<T, E>,
    T: Debug,
    E: Debug,
{
    assert!(n >= 1);

    retry::retry_with_index(Fixed::from_millis(0), |i| {
        let result = match operation() {
            Ok(value) => retry::OperationResult::Ok(value),
            Err(error) => {
                if i < n {
                    retry::OperationResult::Retry(error)
                } else {
                    retry::OperationResult::Err(error)
                }
            }
        };
        match result {
            retry::OperationResult::Ok(_) => info!("{:#?}", result),
            retry::OperationResult::Retry(_) => warn!("{:#?}", result),
            retry::OperationResult::Err(_) => error!("{:#?}", result),
        }
        result
    })
}
