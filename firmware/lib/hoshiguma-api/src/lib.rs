#![cfg_attr(feature = "no-std", no_std)]

#[cfg(feature = "device-cooler")]
pub mod cooler;

pub const RESPONSE_PAYLOAD_CAPACITY: usize = 200;

pub struct Response {
    id: [u8; 4],
    data: heapless::Vec<u8, RESPONSE_PAYLOAD_CAPACITY>,
}
