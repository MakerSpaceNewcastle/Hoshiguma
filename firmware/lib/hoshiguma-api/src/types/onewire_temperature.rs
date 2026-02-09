use crate::TemperatureReading;
use defmt::Format;
use heapless::Vec;
use serde::{Deserialize, Serialize};

pub type OnewireAddress = u64;

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OnewireTemperatureSensorReading {
    pub address: OnewireAddress,
    pub reading: TemperatureReading,
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

impl IntoIterator for OnewireTemperatureSensorReadings {
    type Item = OnewireTemperatureSensorReading;
    type IntoIter = heapless::vec::IntoIter<Self::Item, 8, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
