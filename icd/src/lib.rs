#![cfg_attr(not(feature = "std"), no_std)]

pub mod payload;

use heapless as _;
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type TelemString = std::string::String;
#[cfg(not(feature = "std"))]
pub type TelemString = heapless::String<64>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Message {
    pub millis_since_boot: u64,
    pub payload: payload::Payload,
}
