use super::types::{CompressorState, CoolantPumpState, RadiatorFanState, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetState,

    SetRadiatorFan(RadiatorFanState),
    SetCompressor(CompressorState),
    SetCoolantPump(CoolantPumpState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    GetState(State),

    SetRadiatorFan,
    SetCompressor,
    SetCoolantPump,
}
