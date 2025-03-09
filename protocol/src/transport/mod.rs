use core::time::Duration;

pub trait Server<R, S> {
    async fn receive_message(&mut self, timeout: Duration) -> Result<super::Message<R, S>, ()>;
    async fn send_stream_message(&mut self, data: S) -> Result<(), ()>;
}

pub trait Client<R, S> {
    async fn receive_message(&mut self, timeout: Duration) -> Result<super::Message<R, S>, ()>;
    async fn send_rpc_message(&mut self, msg: R) -> Result<R, ()>;
}

#[cfg(feature = "std")]
pub mod serial_std;

#[cfg(feature = "no-std")]
pub mod embassy_uart_no_std;
