use crate::types::TemperatureReading;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct State {
    temperatures: Temperatures,
    heat_exchange_fluid_level: HeatExchangeFluidLevel,
    coolant_header_tank_level: HeaderTankCoolantLevelReading,
    coolant_flow_rate: CoolantFlow,

    stirrer: Stirrer,
    coolant_pump: CoolantPump,
    radiator_fan: RadiatorFan,
    compressor: Compressor,
}

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

/// The flow of coolant in litres per second.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct CoolantFlow(f64);

impl Deref for CoolantFlow {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CoolantFlow {
    pub fn new(litres: f64, seconds: f64) -> Self {
        Self(litres / seconds)
    }
}
