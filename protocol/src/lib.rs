#![cfg_attr(feature = "no-std", no_std)]

// pub mod payload;
pub mod serial;

use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type String<const N: usize> = std::string::String;
#[cfg(feature = "no-std")]
pub type String<const N: usize> = heapless::String<N>;

#[cfg(feature = "std")]
pub type Vec<T, const N: usize> = std::vec::Vec<T>;
#[cfg(feature = "no-std")]
pub type Vec<T, const N: usize> = heapless::Vec<T, N>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Message<R, S> {
    Rpc(R),
    Stream(Stream<S>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Rpc<REQ, RESP> {
    Request(REQ),
    Response(RESP),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Stream<T> {
    Pub { seq: u32, payload: T },
    Ack { seq: u32 },
}

mod common {
    pub type Ping = super::Rpc<u32, u32>;
    pub type GetVersion = super::Rpc<(), crate::String<16>>;
    pub type GetUptime = super::Rpc<(), u64>;
    pub type Reset = super::Rpc<(), ()>;
}

mod peripheral_controller {
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
