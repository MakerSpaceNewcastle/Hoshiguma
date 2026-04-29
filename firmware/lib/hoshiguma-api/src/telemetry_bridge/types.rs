use defmt::Format;
use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedTelemetryDataPoint(pub String<256>);
