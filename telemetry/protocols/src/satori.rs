use serde::{Deserialize, Serialize};
use enumset::{EnumSet, EnumSetType};

#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    StateChanged(Status),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    temperature: Temperatures,
    coolant_level: CoolantLevel,
    coolant_pump_rpm: f32,
    coolant_flow_rate: f32,

    potential_problems: EnumSet<MachineProblem>,
    problems: EnumSet<MachineProblem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temperatures {
    pub coolant_flow: f32,
    pub coolant_return: f32,

    pub coolant_resevoir_upper: f32,
    pub coolant_resevoir_lower: f32,

    pub coolant_pump: f32,

    pub room_ambient: f32,
    pub laser_bay: f32,
    pub electronics_bay: f32,
}

#[derive(Debug, Serialize, Deserialize)]
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
