use crate::types::TemperatureReading;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Temperatures {
    pub onboard: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_mid: TemperatureReading,
    pub coolant_return: TemperatureReading,

    pub heat_exchange_fluid: TemperatureReading,
    pub heat_exchanger_loop: TemperatureReading,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Compressor {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RadiatorFan {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Stirrer {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantPump {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum HeatExchangeFluidLevel {
    Normal,
    Low,
}

pub type HeaderTankCoolantLevelReading = Result<HeaderTankCoolantLevel, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum HeaderTankCoolantLevel {
    Empty,
    Normal,
    Full,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct CoolantFlow(f64);

impl CoolantFlow {
    pub fn new(litres: f64, seconds: f64) -> Self {
        Self(litres / seconds)
    }
}
