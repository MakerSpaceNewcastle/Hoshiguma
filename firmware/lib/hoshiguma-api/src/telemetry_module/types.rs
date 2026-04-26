use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedTelemetryDataPoint(heapless::String<256>);
