#![cfg_attr(feature = "no-std", no_std)]

use core::time::Duration;
use serde::{Deserialize, Serialize};

#[cfg(feature = "no-std")]
use defmt::{debug, error, trace, warn};
#[cfg(feature = "std")]
use log::{debug, error, trace, warn};

pub mod client;
pub mod server;
pub mod transport;

#[cfg(test)]
mod test;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct RpcMessage<REQ, RESP> {
    seq: u32,
    kind: RpcMessageKind<REQ, RESP>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RpcMessageKind<REQ, RESP> {
    Request { payload: REQ },
    RequestAck,
    Response { payload: RESP },
    ResponseAck,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Error {
    #[error("An RPC request is already being processed")]
    RequestAlreadyInProgress,

    #[error("No RPC request is currently in progress")]
    NoRequestInProgress,

    #[error("The incorrect RPC message type was received")]
    IncorrectMessageType,

    #[error("Incorrect sequence number (expected {expected}, got {actual})")]
    IncorrectSequenceNumber { expected: u32, actual: u32 },

    #[error("No acknowledgement was received")]
    NoAck,

    #[error("Timeout")]
    Timeout,

    #[error("Transport error")]
    TransportError,

    #[error("Serialization error")]
    SerializeError,

    #[error("Deserialization error")]
    DeserializeError,
}

const ACK_TIMEOUT: Duration = Duration::from_millis(100);
