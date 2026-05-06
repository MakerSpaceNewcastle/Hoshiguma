use super::types::{CompressorState, CoolantPumpState, RadiatorFanState};
use crate::{
    MessageId, MessagePayload, SystemInformation, SystemInformationRequestPayload,
    SystemInformationResponsePayload,
};
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

impl SystemInformationRequestPayload for Request {
    fn system_information() -> Self {
        Self::GetSystemInformation
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"clrP"
    }
}

impl SystemInformationResponsePayload for Response {
    fn system_information(self) -> Option<SystemInformation> {
        match self.0 {
            Ok(ResponseData::SystemInformation(info)) => Some(info),
            _ => None,
        }
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    SystemInformation(SystemInformation),

    RadiatorFanState(super::RadiatorFanState),
    CompressorState(super::CompressorState),
    CoolantPumpState(super::CoolantPumpState),
    Temperatures(crate::OnewireTemperatureSensorReadings),
    CoolantFlowRate(super::CoolantRate),
    CoolantReturnRate(super::CoolantRate),
}
