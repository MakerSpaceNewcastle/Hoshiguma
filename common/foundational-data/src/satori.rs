use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type Vec<T> = std::vec::Vec<T>;
#[cfg(not(feature = "std"))]
pub type Vec<T> = heapless::Vec<T, 16>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Payload {
    DiscoveredOneWireDevice{address: u64},
    StateChanged(Status),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    pub temperature: Temperatures,

    pub coolant_level: Option<CoolantLevel>,

    pub coolant_pump_rpm: f32,

    pub coolant_flow_rate: f32,

    pub potential_problems: Vec<PotentialMachineProblem>,
    pub problems: Vec<MachineProblem>,
}

impl From<&Status> for Payload {
    fn from(value: &Status) -> Self {
        Self::StateChanged(value.clone())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Temperatures {
    pub coolant_flow: Option<f32>,
    pub coolant_return: Option<f32>,

    pub coolant_resevoir_upper: Option<f32>,
    pub coolant_resevoir_lower: Option<f32>,

    pub coolant_pump: Option<f32>,

    pub room_ambient: Option<f32>,
    pub laser_bay: Option<f32>,
    pub electronics_bay: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CoolantLevel {
    Full,
    Low,
    CriticallyLow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineProblem {
    pub kind: ProblemKind,
    pub severity: ProblemSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialMachineProblem {
    pub problem: MachineProblem,
    pub since: super::TimeMillis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemSeverity{
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemKind {
    CoolantLevelSensorFault,
    CoolantLevelInsufficient,

    CoolantFlowRateSensorFault,
    CoolantFlowRateInsufficient,

    CoolantPumpSpeedSensorFault,
    CoolantPumpSpeedOutOfSpec,

    TemperatureSensorFault,
    CoolantFlowOvertemperature,
    CoolantReturnOvertemperature,
    ElectronicsBayOvertemperature,
    LaserBayOvertemperature,
}
