use crate::types::TemperatureReading;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct State {
    pub stirrer: StirrerState,
    pub coolant_pump: CoolantPumpState,
    pub compressor: CompressorState,
    pub radiator_fan: RadiatorFanState,

    pub coolant_header_tank_level: HeaderTankCoolantLevelReading,
    pub heat_exchange_fluid_level: HeatExchangeFluidLevel,
    pub coolant_flow_rate: CoolantFlow,
    pub temperatures: Temperatures,
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
pub enum CompressorState {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RadiatorFanState {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum StirrerState {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantPumpState {
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

/// The flow of coolant in litres per minute.
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
    pub const ZERO: Self = Self(0.0);

    pub fn new(litres_per_minute: f64) -> Self {
        Self(litres_per_minute)
    }
}
