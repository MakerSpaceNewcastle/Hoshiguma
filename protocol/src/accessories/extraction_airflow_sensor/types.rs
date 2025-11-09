use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Measurement {
    pub differential_pressure: f32,
    pub temperature: f32,
}

pub type FallibleMeasurement = Result<Measurement, ()>;
