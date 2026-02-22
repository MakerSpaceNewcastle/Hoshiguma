use super::types::{CompressorState, CoolantPumpState, RadiatorFanState};
use crate::ResponsePayload;
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,
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
    GitRevision(crate::GitRevisionString),
    Uptime(Duration),
    BootReason(crate::BootReason),
    RadiatorFanState(super::RadiatorFanState),
    CompressorState(super::CompressorState),
    CoolantPumpState(super::CoolantPumpState),
    Tempreatures(super::TemperatureReadings),
    CoolantFlowRate(super::CoolantRate),
    CoolantReturnRate(super::CoolantRate),
}
