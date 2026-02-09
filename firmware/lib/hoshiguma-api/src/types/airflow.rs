use defmt::Format;
use serde::{Deserialize, Serialize};

pub type AirflowSensorMeasurement = Result<AirflowSensorMeasurementInner, ()>;

#[derive(Default, Debug, Format, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirflowSensorMeasurementInner {
    pub differential_pressure: f32,
    pub temperature: f32,
}
