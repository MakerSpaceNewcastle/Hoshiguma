#![cfg_attr(not(feature = "std"), no_std)]

// pub mod payload;

use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type String<const N: usize> = std::string::String;
#[cfg(not(feature = "std"))]
pub type String<const N: usize> = heapless::String<N>;

#[cfg(feature = "std")]
pub type Vec<T, const N: usize> = std::vec::Vec<T>;
#[cfg(not(feature = "std"))]
pub type Vec<T, const N: usize> = heapless::Vec<T, N>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub millis_since_boot: u64,
    pub msg: Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Rpc(controller::Rpc),
    Publish,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RpcMessage<REQ, RES> {
    Request(REQ),
    Response(RES),
}

mod controller {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum Rpc {
        Ping(super::RpcMessage<u32, u32>),
        GetVersion(super::RpcMessage<(), crate::String<16>>),
    }
}
