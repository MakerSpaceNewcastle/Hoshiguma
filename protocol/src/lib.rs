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
pub enum Message<RPC, PUB> {
    Rpc(RPC),
    Publish { millis_since_boot: u64, msg: PUB },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rpc<REQ, RES> {
    Request(REQ),
    Response(RES),
}

mod common {
    pub type Ping = super::Rpc<u32, u32>;
    pub type GetVersion = super::Rpc<(), crate::String<16>>;
    pub type GetUptime = super::Rpc<(), u64>;
    pub type Reset = super::Rpc<(), ()>;
}

mod controller {
    use serde::{Deserialize, Serialize};

    pub type ControllerMessage = super::Message<Rpc, ()>;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum Rpc {
        Ping(crate::common::Ping),
        GetVersion(crate::common::GetVersion),
        GetUptime(crate::common::GetUptime),
        Reset(crate::common::Reset),
    }
}
