use defmt::Format;
use getset::Getters;
use heapless::Vec;
use serde::{Deserialize, Serialize};

pub type OnewireAddress = u64;

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Getters)]
pub struct OnewireTemperatureSensorReading {
    address: OnewireAddress,
    reading: Result<f32, ()>,
}

impl OnewireTemperatureSensorReading {
    pub fn new(address: OnewireAddress, reading: Result<f32, ()>) -> Self {
        Self { address, reading }
    }
}

#[derive(Debug, Format, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnewireTemperatureSensorReadings(Vec<OnewireTemperatureSensorReading, 8>);

impl OnewireTemperatureSensorReadings {
    pub const MAX_NUM_SENSORS: usize = 8;

    pub fn push(
        &mut self,
        reading: OnewireTemperatureSensorReading,
    ) -> Result<(), OnewireTemperatureSensorReading> {
        self.0.push(reading).map_err(|_| reading)
    }
}
