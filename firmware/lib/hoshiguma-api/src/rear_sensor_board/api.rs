use crate::{MessageId, MessagePayload};
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

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"rsbP"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    SystemInformation(crate::SystemInformation),

    StatusLightSettings(super::StatusLightSettings),
    ExtractionAriflow(crate::AirflowSensorMeasurement),
    Temperatures(crate::OnewireTemperatureSensorReadings),
}
