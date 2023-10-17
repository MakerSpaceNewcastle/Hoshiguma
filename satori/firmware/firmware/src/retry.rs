use log::warn;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failed after {attempts} attempts")]
pub struct RetryError<E> {
    attempts: usize,
    last_error: E,
}

pub(crate) fn retry<T, E>(
    attempts: usize,
    mut f: impl FnMut() -> Result<T, E>,
) -> Result<T, RetryError<E>> {
    let mut err = None;

    for attempt in 0..attempts {
        match f() {
            Ok(r) => return Ok(r),
            Err(e) => {
                let attempt = attempt + 1;
                warn!("Failed on attempt {attempt} out of {attempts}");
                err = Some(e);
            }
        }
    }

    Err(RetryError {
        attempts,
        last_error: err.unwrap(),
    })
}
