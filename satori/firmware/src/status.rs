use enumset::{EnumSet, EnumSetType};
use serde::Deserialize;

type String = heapless::String<32>;
type Vec<T> = heapless::Vec<T, 8>;
type TimeMillis = u32;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Message {
    time: TimeMillis,
    iteration_id: Option<u32>,
    pub payload: Payload,
}

#[derive(Debug, Deserialize)]
pub enum Payload {
    Boot(Boot),
    Panic(Panic),
    StatusChanged(Status),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Boot {
    name: String,
    pub git_revision: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Panic {
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Status {
    temperature: Temperatures,
    coolant_level: CoolantLevel,
    coolant_pump_rpm: f32,
    coolant_flow_rate: f32,

    potential_problems: EnumSet<MachineProblem>,
    problems: EnumSet<MachineProblem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Temperatures {
    pub coolant_flow: f32,
    pub coolant_return: f32,

    pub coolant_resevoir_upper: f32,
    pub coolant_resevoir_lower: f32,

    pub coolant_pump: f32,

    pub room_ambient: f32,
    pub laser_bay: f32,
    pub electronics_bay: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) enum CoolantLevel {
    Full,
    Low,
    CriticallyLow,
}

#[derive(Debug, Deserialize, EnumSetType)]
enum MachineProblem {
    CoolentLevelInsufficient,
    CoolantFlowRateInsufficient,
    CoolantFlowOvertemperature,
    CoolantReturnOvertemperature,
    ElectronicsBayOvertemperature,
    LaserBayOvertemperature,
}
