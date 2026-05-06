use crate::{
    MessageId, MessagePayload, SystemInformation, SystemInformationRequestPayload,
    SystemInformationResponsePayload,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetSystemInformation,

    SetStatusLight(super::StatusLightSettings),
    GetExtractionAirflow,
    GetTemperatures,
}

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"rsbQ"
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
        b"rsbP"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    SystemInformation(SystemInformation),

    StatusLightSettings(super::StatusLightSettings),
    ExtractionAriflow(crate::AirflowSensorMeasurement),
    Temperatures(crate::OnewireTemperatureSensorReadings),
}

impl SystemInformationResponsePayload for ResponseData {
    fn system_information(self) -> Option<SystemInformation> {
        match self {
            ResponseData::SystemInformation(info) => Some(info),
            _ => None,
        }
    }
}
