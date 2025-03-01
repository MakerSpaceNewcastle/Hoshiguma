#![cfg_attr(not(feature = "std"), no_std)]

pub mod payload;

use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type String = std::string::String;
#[cfg(feature = "no-std")]
pub type String = heapless::String<64>;

#[cfg(feature = "std")]
pub type Vec<T, const N: usize>= std::vec::Vec<T>;
#[cfg(not(feature = "std"))]
pub type Vec<T, const N: usize>= heapless::Vec<T, N>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
#[allow(dead_code)]
pub struct Message {
    pub millis_since_boot: u64,
    pub payload: payload::Payload,
}
