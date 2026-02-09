use crate::{OnewireAddress, OnewireTemperatureSensorReading};
use defmt::Format;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter};

pub type TemperatureReading = Result<f32, ()>;

#[derive(
    Debug,
    Format,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    EnumIter,
    EnumCount,
)]
pub enum TemperatureSensor {
    OrchastratorPcb,

    CoolerPcb,
    CoolantReservoir,

    CoolantFlowAtTube,
    CoolantReturnAtTube,

    UnknownOnewire(OnewireAddress),
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemperatureSensorReading {
    pub sensor: TemperatureSensor,
    pub reading: TemperatureReading,
}

impl From<OnewireTemperatureSensorReading> for TemperatureSensorReading {
    fn from(value: OnewireTemperatureSensorReading) -> Self {
        Self {
            sensor: TemperatureSensor::UnknownOnewire(value.address),
            reading: value.reading,
        }
    }
}
