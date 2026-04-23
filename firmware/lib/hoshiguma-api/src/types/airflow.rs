use defmt::Format;
use serde::{Deserialize, Serialize};

pub type AirflowSensorMeasurement = Result<AirflowSensorMeasurementInner, ()>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirflowSensorMeasurementInner {
    pub differential_pressure: f32,
    pub temperature: f32,
}
