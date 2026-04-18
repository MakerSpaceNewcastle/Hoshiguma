use core::ops::Deref;
use serde::{Deserialize, Serialize};

pub type OnewireTemperatureSensorReadings = crate::OnewireTemperatureSensorReadings<8>;

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoolantPumpState {
    Idle,
    Run,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressorState {
    Idle,
    Run,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum RadiatorFanState {
    Idle,
    Run,
}

/// The rate of flow of coolant in litres per minute.
#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoolantRate(f64);

impl Deref for CoolantRate {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CoolantRate {
    pub const ZERO: Self = Self(0.0);

    pub fn new(litres_per_minute: f64) -> Self {
        Self(litres_per_minute)
    }
}
