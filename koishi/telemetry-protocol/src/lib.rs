use serde::Deserialize;
use enumset::{EnumSetType, EnumSet};

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

    InputsChanged(Inputs),
    OutputsChanged(Outputs),

    MachineStatusChanged(MachineStatus),
    AirAssistStatusChanged(AirAssistStatus),
    ExtractionStatusChanged(ExtractionStatus),
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
#[allow(dead_code)]
pub struct Inputs {
    doors_closed: bool,
    cooling_ok: bool,
    machine_running: bool,
    air_pump_demand: bool,
    extraction_mode: ExtractionMode,
}

#[derive(Debug, Deserialize)]
pub enum ExtractionMode {
    Normal,
    Run,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Outputs {
    controller_machine_alarm: AlarmState,
    controller_cooling_alarm: AlarmState,
    laser_enable: bool,
    status_light: StatusLight,
    air_pump: bool,
    extractor_fan: bool,
}

#[derive(Debug, Deserialize)]
pub enum AlarmState {
    Normal,
    Alarm,
}

#[derive(Debug, Deserialize)]
pub enum StatusLight {
    Green,
    Amber,
    Red,
}

#[derive(Debug, Deserialize)]
pub enum MachineStatus {
    Running,
    Idle,
    Problem(EnumSet<MachineProblem>),
}

#[derive(Debug, Deserialize, EnumSetType)]
pub enum MachineProblem {
    DoorOpen,
    External,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AirAssistStatus {
    state: RunOnDelay,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExtractionStatus {
    state: RunOnDelay,
    r#override: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RunOnDelay {
    delay: TimeMillis,
    state: RunOnDelayState,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub enum RunOnDelayState {
    Demand,
    RunOn { end: TimeMillis },
    Idle,
}
