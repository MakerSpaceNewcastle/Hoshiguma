use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Monitor {
    LogicPowerSupplyNotPresent,

    ChassisIntrusion,

    CoolantResevoirLevelSensorFault,
    CoolantResevoirLevel,

    TemperatureSensorFault,
    CoolantFlowTemperature,
    CoolantResevoirTemperature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MonitorState {
    Normal,
    Warn,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorStatus {
    pub since_millis: u64,
    pub monitor: Monitor,
    pub state: MonitorState,
}
