use defmt::Format;
use getset::Getters;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub type GitRevisionString = String<20>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

pub type OnewireAddress = u64;

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Getters)]
pub struct OnewireTemperatureSensorReading {
    device: OnewireAddress,
    reading: Result<f32, ()>,
}

impl OnewireTemperatureSensorReading {
    pub fn new(device: OnewireAddress, reading: Result<f32, ()>) -> Self {
        Self { device, reading }
    }
}

#[derive(Debug, Format, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnewireTemperatureSensorReadings<const MAX_SENSORS: usize>(
    Vec<OnewireTemperatureSensorReading, MAX_SENSORS>,
);

impl<const MAX_SENSORS: usize> OnewireTemperatureSensorReadings<MAX_SENSORS> {
    pub fn push(
        &mut self,
        reading: OnewireTemperatureSensorReading,
    ) -> Result<(), OnewireTemperatureSensorReading> {
        self.0.push(reading).map_err(|_| reading)
    }
}
