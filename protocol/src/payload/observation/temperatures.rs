use serde::{Deserialize, Serialize};

pub type TemperatureReading = Result<f32, ()>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Temperatures {
    pub onboard: TemperatureReading,
    pub electronics_bay_top: TemperatureReading,

    pub laser_chamber: TemperatureReading,

    pub ambient: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_return: TemperatureReading,

    pub coolant_resevoir_bottom: TemperatureReading,
    pub coolant_resevoir_top: TemperatureReading,

    pub coolant_pump: TemperatureReading,
}
