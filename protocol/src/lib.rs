#![cfg_attr(feature = "no-std", no_std)]

pub mod common;
pub mod peripheral_controller;
pub mod serial;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Message<R, S> {
    Rpc(R),
    Stream(StreamMessage<S>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RpcMessage<REQ, RESP> {
    Request(REQ),
    Response(RESP),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum StreamMessage<T> {
    Pub { seq: u32, payload: T },
    Ack { seq: u32 },
}

impl<T> StreamMessage<T> {
    pub fn get_ack(&self) -> Self {
        match self {
            Self::Pub { seq, payload: _ } => Self::Ack { seq: *seq },
            _ => panic!("should not ack an ack"),
        }
    }
}
