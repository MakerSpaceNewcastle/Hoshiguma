mod config;
pub use config::*;

mod client_request;
pub use client_request::*;

mod helpers;
pub use helpers::*;

mod message_handler;
pub use message_handler::*;

use defmt::Format;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotConnected,
    ConnectionReset,
    SocketReadEof,
    SocketWrite,
    MessageDeserialize,
    MessageSerialize,
}
