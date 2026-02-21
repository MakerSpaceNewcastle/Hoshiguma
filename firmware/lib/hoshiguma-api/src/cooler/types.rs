use core::ops::Deref;
use heapless::Vec;
use hoshiguma_telemetry::{AsTelemetry, TelemetryDataPoint, TelemetryStrValue, TelemetryValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct State {
    pub coolant_pump: CoolantPumpState,
    pub compressor: CompressorState,
    pub radiator_fan: RadiatorFanState,

    pub coolant_reservoir_level: CoolantReservoirLevel,
    pub coolant_flow_rate: CoolantFlow,
    pub temperatures: Temperatures,
}

pub type TemperatureReading = Result<f32, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Temperatures {
    pub onboard: TemperatureReading,
    pub internal_ambient: TemperatureReading,

    pub coolant_pump_motor: TemperatureReading,

    pub reservoir: TemperatureReading,
}

impl AsTelemetry<&str, 8> for Temperatures {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 8> {
        let mut v = Vec::new();

        for (name, reading) in [
            ("temperature,sensor=cooler_onboard", self.onboard),
            ("temperature,sensor=cooler_internal", self.internal_ambient),
            ("temperature,sensor=coolant_pump", self.coolant_pump_motor),
            ("temperature,sensor=coolant_reservoir", self.reservoir),
        ] {
            v.push(TelemetryDataPoint {
                measurement: name,
                field: "sensor_ok",
                value: TelemetryValue::Bool(reading.is_ok()),
                timestamp: None,
            })
            .unwrap();

            if let Ok(reading) = reading {
                v.push(TelemetryDataPoint {
                    measurement: name,
                    field: "value",
                    value: TelemetryValue::Float32(reading),
                    timestamp: None,
                })
                .unwrap();
            }
        }

        v
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantPumpState {
    Idle,
    Run,
}

impl AsTelemetry<&str, 1> for CoolantPumpState {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 1> {
        Vec::from_array([TelemetryDataPoint {
            measurement: "coolant_pump",
            field: "value",
            value: TelemetryValue::String(self.telemetry_str()),
            timestamp: None,
        }])
    }
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

impl AsTelemetry<&str, 1> for CompressorState {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 1> {
        Vec::from_array([TelemetryDataPoint {
            measurement: "cooler_compressor",
            field: "value",
            value: TelemetryValue::String(self.telemetry_str()),
            timestamp: None,
        }])
    }
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

impl AsTelemetry<&str, 1> for RadiatorFanState {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 1> {
        Vec::from_array([TelemetryDataPoint {
            measurement: "cooler_radiator_fan",
            field: "value",
            value: TelemetryValue::String(self.telemetry_str()),
            timestamp: None,
        }])
    }
}

impl TelemetryStrValue for RadiatorFanState {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantReservoirLevel {
    Normal,
    Low,
}

impl AsTelemetry<&'static str, 1> for CoolantReservoirLevel {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 1> {
        Vec::from_array([TelemetryDataPoint {
            measurement: "coolant_reservoir_level",
            field: "value",
            value: TelemetryValue::String(self.telemetry_str()),
            timestamp: None,
        }])
    }
}

impl TelemetryStrValue for CoolantReservoirLevel {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Low => "low",
        }
    }
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

impl AsTelemetry<&'static str, 1> for CoolantFlow {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<&'static str>, 1> {
        Vec::from_array([TelemetryDataPoint {
            measurement: "coolant_rate",
            field: "flow",
            value: TelemetryValue::Float64(**self),
            timestamp: None,
        }])
    }
}
