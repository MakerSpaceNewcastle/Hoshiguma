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

use core::time::Duration;
use defmt::Format;
use heapless::String;
use serde::{Deserialize, Serialize};

pub trait TelemetryString {
    fn telemetry_str(&self) -> &'static str;
}

pub type GitRevisionString = String<20>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInformation {
    pub git_revision: GitRevisionString,
    pub uptime: Duration,
    pub boot_reason: BootReason,
}

impl TelemetryString for BootReason {
    fn telemetry_str(&self) -> &'static str {
        match self {
            BootReason::Normal => "normal",
            BootReason::WatchdogTimeout => "watchdog_timeout",
            BootReason::WatchdogForced => "watchdog_forced",
        }
    }
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
