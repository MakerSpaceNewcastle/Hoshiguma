use super::types::{CompressorState, CoolantPumpState, RadiatorFanState};
use crate::{MessageId, MessagePayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetSystemInformation,

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

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"clrQ"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"clrP"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    SystemInformation(crate::SystemInformation),

    RadiatorFanState(super::RadiatorFanState),
    CompressorState(super::CompressorState),
    CoolantPumpState(super::CoolantPumpState),
    Temperatures(crate::OnewireTemperatureSensorReadings),
    CoolantFlowRate(super::CoolantRate),
    CoolantReturnRate(super::CoolantRate),
}
