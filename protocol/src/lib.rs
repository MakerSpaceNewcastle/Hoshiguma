#![cfg_attr(feature = "no-std", no_std)]

pub mod payload;
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
pub struct Message {
    pub millis_since_boot: u64,
    pub payload: payload::Payload,
}
