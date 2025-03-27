use core::time::Duration;

#[allow(async_fn_in_trait)]
pub trait Transport<M> {
    async fn flush(&mut self, timeout: Duration) -> Result<usize, crate::Error>;
    async fn receive_message(&mut self, timeout: Duration) -> Result<M, crate::Error>;
    async fn transmit_message(&mut self, msg: M) -> Result<(), crate::Error>;
}

#[cfg(feature = "std")]
pub mod serialport;

#[cfg(feature = "std")]
pub mod tokio_channels;

#[cfg(feature = "no-std")]
pub mod embedded;
