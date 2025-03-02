use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MonitorState {
    Normal,
    Warn,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct MonitorStatus {
    pub since_millis: u64,
    pub monitor: Monitor,
    pub state: MonitorState,
}
