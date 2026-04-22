use defmt::Format;
use serde::{Deserialize, Serialize};

pub const NUM_ONEWIRE_TEMPERATURE_SENSORS: usize = 8;

pub type OnewireTemperatureSensorReadings =
    crate::OnewireTemperatureSensorReadings<NUM_ONEWIRE_TEMPERATURE_SENSORS>;

pub type AirflowSensorMeasurement = Result<AirflowSensorMeasurementInner, ()>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirflowSensorMeasurementInner {
    pub differential_pressure: f32,
    pub temperature: f32,
}
