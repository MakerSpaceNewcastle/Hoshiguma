use core::ops::Deref;
use hoshiguma_telemetry::TelemetryStrValue;
use serde::{Deserialize, Serialize};

pub type TemperatureReadings = crate::TemperatureReadings<8>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantPumpState {
    Idle,
    Run,
}

impl TelemetryStrValue for CoolantPumpState {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CompressorState {
    Idle,
    Run,
}

impl TelemetryStrValue for CompressorState {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RadiatorFanState {
    Idle,
    Run,
}

impl TelemetryStrValue for RadiatorFanState {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

/// The rate of flow of coolant in litres per minute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
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
