use super::types::{CompressorState, CoolantPumpState, RadiatorFanState, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    SetRadiatorFan(RadiatorFanState),
    SetCompressor(CompressorState),
    SetCoolantPump(CoolantPumpState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Response(Result<Option<ResponseData>, ()>);

impl Response {
    pub const ID: &[u8; 4] = b"c00l";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ResponseData {
    GetState(State),
}
