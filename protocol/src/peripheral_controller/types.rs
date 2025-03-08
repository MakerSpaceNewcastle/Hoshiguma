use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistDemand {
    Idle,
    Demand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistPump {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionFan {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum LaserEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StatusLamp {
    pub red: bool,
    pub amber: bool,
    pub green: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Monitor {
    LogicPowerSupplyNotPresent,

    ChassisIntrusion,

    CoolantResevoirLevelSensorFault,
    CoolantResevoirLevel,

    TemperatureSensorFault,
    CoolantFlowTemperature,
    CoolantResevoirTemperature,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MonitorState {
    Normal,
    Warn,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct MonitorStatus {
    pub since_millis: u64,
    pub monitor: Monitor,
    pub state: MonitorState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct ActiveAlarms {
    pub alarms: Vec<MonitorStatus, 16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ChassisIntrusion {
    Normal,
    Intruded,
}

pub type CoolantResevoirLevelReading = Result<CoolantResevoirLevel, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachinePower {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineRun {
    Idle,
    Running,
}

pub type TemperatureReading = Result<f32, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Temperatures {
    pub onboard: TemperatureReading,
    pub electronics_bay_top: TemperatureReading,

    pub laser_chamber: TemperatureReading,

    pub ambient: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_return: TemperatureReading,

    pub coolant_resevoir_bottom: TemperatureReading,
    pub coolant_resevoir_top: TemperatureReading,

    pub coolant_pump: TemperatureReading,
}
