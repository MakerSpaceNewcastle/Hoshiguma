use log::warn;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failed after {N} attempts")]
pub struct RetryError<const N: usize, E> {
    last_error: E,
}

pub(crate) fn retry<const N: usize, T, E>(
    mut f: impl FnMut() -> Result<T, E>,
) -> Result<T, RetryError<N, E>> {
    let mut err = None;

    for attempt in 0..N {
        match f() {
            Ok(r) => return Ok(r),
            Err(e) => {
                let attempt = attempt + 1;
                warn!("Failed on attempt {attempt} out of {N}");
                err = Some(e);
            }
        }
    }

    Err(RetryError {
        last_error: err.unwrap(),
    })
}
