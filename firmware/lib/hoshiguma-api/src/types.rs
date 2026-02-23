use getset::Getters;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub type GitRevisionString = String<20>;

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

pub type OnewireAddress = u64;

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct OnewireTemperatureSensorReading {
    device: OnewireAddress,
    reading: Result<f32, ()>,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnewireTemperatureSensorReadings<const MAX_SENSORS: usize> {
    readings: Vec<OnewireTemperatureSensorReading, MAX_SENSORS>,
}
