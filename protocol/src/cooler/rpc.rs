use super::{
    event::Event,
    types::{Compressor, CoolantPump, RadiatorFan, Stirrer},
};
use crate::{event_queue::EventStatistics, types::SystemInformation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    Ping(u32),
    GetSystemInformation,

    GetEventCount,
    GetEventStatistics,
    GetOldestEvent,

    SetRadiatorFan(RadiatorFan),
    SetCompressor(Compressor),
    SetStirrer(Stirrer),
    SetCoolantPump(CoolantPump),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    Ping(u32),
    GetSystemInformation(SystemInformation),

    GetEventCount(usize),
    GetEventStatistics(EventStatistics),
    GetOldestEvent(Option<Event>),

    SetRadiatorFan,
    SetCompressor,
    SetStirrer,
    SetCoolantPump,
}
