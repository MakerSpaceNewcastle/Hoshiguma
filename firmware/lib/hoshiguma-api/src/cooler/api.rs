use super::types::{CompressorState, CoolantPumpState, RadiatorFanState};
use crate::ResponsePayload;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetRadiatorFanState,
    SetRadiatorFanState(RadiatorFanState),
    GetCompressorState,
    SetCompressorState(CompressorState),
    GetCoolantPumpState,
    SetCoolantPumpState(CoolantPumpState),
    GetTemperatures,
    GetCoolantFlowRate,
    GetCoolantReturnRate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Response(Result<Option<ResponseData>, ()>);

impl ResponsePayload for Response {
    fn id() -> &'static [u8; 4] {
        b"c00l"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ResponseData {
    RadiatorFanState,
    CompressorState,
    CoolantPumpState,
    Tempreatures,
    CoolantFlowRate,
    CoolantReturnRate,
}
