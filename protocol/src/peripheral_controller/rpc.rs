use super::event::Event;
use crate::{event_queue::EventStatistics, types::SystemInformation};
use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    Ping(u32),
    GetSystemInformation,
    GetEventCount,
    GetEventStatistics,
    GetOldestEvents(usize),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    Ping(u32),
    GetSystemInformation(SystemInformation),
    GetEventCount(usize),
    GetEventStatistics(EventStatistics),
    GetOldestEvents(Vec<Event, EVENTS_PER_MESSAGE>),
}

pub const EVENTS_PER_MESSAGE: usize = 8;
