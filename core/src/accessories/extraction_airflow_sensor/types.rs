use crate::telemetry::{AsTelemetry, StaticTelemetryDataPoint, TelemetryValue};
use core::ops::Deref;
use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct MeasurementInner {
    pub differential_pressure: f32,
    pub temperature: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Measurement(Result<MeasurementInner, ()>);

impl Measurement {
    pub fn new(inner: Result<MeasurementInner, ()>) -> Self {
        Self(inner)
    }
}

impl Deref for Measurement {
    type Target = Result<MeasurementInner, ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsTelemetry<4, 3> for Measurement {
    fn strings() -> [&'static str; 4] {
        [
            "fume_extraction_airflow",
            "ok",
            "value",
            "sensor_temperature",
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 3> {
        let mut v = Vec::new();

        v.push(StaticTelemetryDataPoint {
            measurement: "fume_extraction_airflow",
            field: "ok",
            value: TelemetryValue::Bool(self.0.is_ok()),
            timestamp_nanoseconds: None,
        })
        .unwrap();

        if let Ok(m) = &self.0 {
            v.push(StaticTelemetryDataPoint {
                measurement: "fume_extraction_airflow",
                field: "value",
                value: TelemetryValue::Float32(m.differential_pressure),
                timestamp_nanoseconds: None,
            })
            .unwrap();

            v.push(StaticTelemetryDataPoint {
                measurement: "fume_extraction_airflow",
                field: "sensor_temperature",
                value: TelemetryValue::Float32(m.temperature),
                timestamp_nanoseconds: None,
            })
            .unwrap();
        }

        v
    }
}
