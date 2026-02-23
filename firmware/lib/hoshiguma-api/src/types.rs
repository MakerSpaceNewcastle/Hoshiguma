use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub type GitRevisionString = String<20>;

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

pub type TemperatureReading = Result<f32, ()>;

// TODO
#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureReadings<const MAX_SENSORS: usize> {
    readings: Vec<(u32, TemperatureReading), MAX_SENSORS>,
}
