use crate::{MessageId, MessagePayload};
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,

    SetStatusLight(super::StatusLightSettings),
    GetExtractionAirflow,
    GetTemperatures,
}

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"rsbq"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"rsbp"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    GitRevision(crate::GitRevisionString),
    Uptime(Duration),
    BootReason(crate::BootReason),

    StatusLightSettings(super::StatusLightSettings),
    ExtractionAriflow(crate::AirflowSensorMeasurement),
    Temperatures(crate::OnewireTemperatureSensorReadings),
}
