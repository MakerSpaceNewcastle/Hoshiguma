mod access_control;
pub use access_control::*;

mod airflow;
pub use airflow::*;

mod hmi;
pub use hmi::*;

mod monitors;
pub use monitors::*;

mod onewire_temperature;
pub use onewire_temperature::*;

mod system;
pub use system::*;

use defmt::Format;
use serde::{Deserialize, Serialize};

pub trait TelemetryString {
    fn telemetry_str(&self) -> &'static str;
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoolingEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoolingDemand {
    Idle,
    Demand,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum AirAssistDemand {
    Idle,
    Demand,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum AirAssistPump {
    Idle,
    Run,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum FumeExtractionFan {
    Idle,
    Run,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum LaserEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusLamp {
    pub red: bool,
    pub amber: bool,
    pub green: bool,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChassisIntrusion {
    Normal,
    Intruded,
}

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachinePower {
    On,
    Off,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineRun {
    Idle,
    Running,
}
