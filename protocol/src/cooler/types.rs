use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct State {
    // Sensors
    // TODO

    // Outputs
    pub radiator_fan_running: bool,
    pub compressor_running: bool,
    pub stirrer_running: bool,
    pub coolant_pump_running: bool,
}
