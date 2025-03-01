#![cfg_attr(not(feature = "std"), no_std)]

// pub mod payload;

use heapless as _;
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type TelemString = std::string::String;
#[cfg(not(feature = "std"))]
pub type TelemString = heapless::String<64>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
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
        GetVersion(super::RpcMessage<(), crate::TelemString>),
    }
}
