use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Payload {
    StateChanged(Status),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    pub temperature: Temperatures,
    pub coolant_level: Option<CoolantLevel>,
    pub coolant_pump_rpm: f32,
    pub coolant_flow_rate: f32,
    // TODO
    // potential_problems: EnumSet<MachineProblem>,
    // problems: EnumSet<MachineProblem>,
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

#[derive(Debug, Serialize, Deserialize, EnumSetType)]
pub enum MachineProblem {
    CoolentLevelInsufficient,
    CoolantFlowRateInsufficient,
    CoolantFlowOvertemperature,
    CoolantReturnOvertemperature,
    ElectronicsBayOvertemperature,
    LaserBayOvertemperature,
}
