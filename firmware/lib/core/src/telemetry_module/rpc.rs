use crate::telemetry::TelemetryDataPoint;
use chrono::{DateTime, Utc};
use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    IsReady,

    GetWallTime,

    GetStringRegistryMetadata,
    ClearStringRegistry,
    PushStringToRegistry(String<{ super::STRING_REGISTRY_MAX_STRING_LENGTH }>),

    SendTelemetryDataPoint(TelemetryDataPoint<usize>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    IsReady(bool),

    GetWallTime(Result<DateTime<Utc>, ()>),

    GetStringRegistryMetadata(crate::string_registry::Metadata),
    ClearStringRegistry,
    PushStringToRegistry(crate::string_registry::Result<()>),

    SendTelemetryDataPoint(Result<(), ()>),
}
