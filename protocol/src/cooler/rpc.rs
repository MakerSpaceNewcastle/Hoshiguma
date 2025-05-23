use super::types::{CompressorState, CoolantPumpState, RadiatorFanState, State};
use crate::types::SystemInformation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    Ping(u32),
    GetSystemInformation,

    GetState,

    SetRadiatorFan(RadiatorFanState),
    SetCompressor(CompressorState),
    SetCoolantPump(CoolantPumpState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    Ping(u32),
    GetSystemInformation(SystemInformation),

    GetState(State),

    SetRadiatorFan,
    SetCompressor,
    SetCoolantPump,
}
