use crate::types::SystemInformation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetSystemInformation,
    GetStringsMetadata,
    GetString(usize),
    GetTelemetryDataPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    GetSystemInformation(SystemInformation),
    GetStringsMetadata(StringsMetadata),
    GetString(Option<StaticTelemetryString>),
    GetTelemetryDataPoint(TelemetryDataPointResponse),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StringsMetadata {
    count: usize,
    checksum: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct TelemetryDataPointResponse {
    data_point: Option<TelemetryDataPoint>,
    more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct TelemetryDataPoint {
    measurement_string_idx: usize,
    field_string_idx: usize,
    value: TelemetryValue,
    timestamp_milliseconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum TelemetryValue {
    Float32(f32),
    String(TelemetryStringValue),
}

pub type StaticTelemetryString = heapless::String<64>;
pub type TelemetryStringValue = heapless::String<32>;
