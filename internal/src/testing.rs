use log::{error, info, warn};
use retry::{delay::Fixed, retry_with_index, OperationResult};
use std::fmt::Debug;

pub fn retry<O, T, E>(n: u64, operation: O) -> Result<T, retry::Error<E>>
where
    O: Fn() -> std::result::Result<T, E>,
    T: Debug,
    E: Debug,
{
    assert!(n >= 1);

    retry_with_index(Fixed::from_millis(0), |i| {
        let result = match operation() {
            Ok(value) => OperationResult::Ok(value),
            Err(error) => {
                if i < n {
                    OperationResult::Retry(error)
                } else {
                    OperationResult::Err(error)
                }
            }
        };
        match result {
            OperationResult::Ok(_) => info!("{:#?}", result),
            OperationResult::Retry(_) => warn!("{:#?}", result),
            OperationResult::Err(_) => error!("{:#?}", result),
        }
        result
    })
}
