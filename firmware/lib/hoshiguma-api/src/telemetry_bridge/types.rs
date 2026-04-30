use defmt::Format;
use heapless::String;
use serde::{Deserialize, Serialize};

pub const TELEMETRY_DATA_POINT_MAX_LEN: usize = 256;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedTelemetryDataPoint(pub String<TELEMETRY_DATA_POINT_MAX_LEN>);
