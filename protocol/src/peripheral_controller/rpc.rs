use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    Ping(u32),
    GetEventCount,
    GetEvents,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    Ping(u32),
    GetEventCount(usize),
    GetEvents,
}
