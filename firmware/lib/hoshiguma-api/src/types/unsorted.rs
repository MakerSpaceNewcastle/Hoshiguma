use defmt::Format;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Debug, Format, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum Interlock {
    /// Normal operation
    OperationPermitted,
    /// The current job may be finished, but no new jobs may be started
    OperationPermittedUntilIdle,
    /// The current job is stopped, no jobs may be started
    OperationDenied,
    /// The machine is unable to be powered on
    MachineProtected,
}

#[derive(
    Debug, Format, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum InterlockAction {
    Normal,
    Disable,
    Shutdown,
}

#[derive(
    Debug, Format, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum DesiredMachinePower {
    Off,
    On,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum CoolingEnable {
    Inhibit,
    Enable,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum CoolingDemand {
    Idle,
    Demand,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum AirAssistDemand {
    Idle,
    Demand,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum AirAssistPump {
    Idle,
    Run,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum FumeExtractionFan {
    Idle,
    Run,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum LaserEnable {
    Inhibit,
    Enable,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum MachineEnable {
    Inhibit,
    Enable,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum Doors {
    Closed,
    Open,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum AcBusPower {
    On,
    Off,
}

#[derive(
    Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum MachineRun {
    Idle,
    Running,
}
