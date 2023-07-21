use serde::Deserialize;

type TimeMillis = u32;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Message {
    time: TimeMillis,
    iteration_id: Option<u32>,
    pub payload: Payload,
}

#[derive(Debug, Deserialize)]
pub(crate) enum Payload {
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
pub(crate) struct Boot {
    name: String,
    pub git_revision: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Panic {
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Inputs {
    doors_closed: bool,
    cooling_ok: bool,
    machine_running: bool,
    air_pump_demand: bool,
    extraction_mode: ExtractionMode,
}

#[derive(Debug, Deserialize)]
pub(crate) enum ExtractionMode {
    Normal,
    Run,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Outputs {
    controller_machine_alarm: AlarmState,
    controller_cooling_alarm: AlarmState,
    laser_enable: bool,
    status_light: StatusLight,
    air_pump: bool,
    extractor_fan: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) enum AlarmState {
    Normal,
    Alarm,
}

#[derive(Debug, Deserialize)]
pub(crate) enum StatusLight {
    Green,
    Amber,
    Red,
}

#[derive(Debug, Deserialize)]
pub(crate) enum MachineStatus {
    Running,
    Idle,
    Problem(MachineProblem),
}

#[derive(Debug, Deserialize)]
pub(crate) enum MachineProblem {
    DoorOpen,
    CoolingFault,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct AirAssistStatus {
    state: RunOnDelay,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct ExtractionStatus {
    state: RunOnDelay,
    r#override: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct RunOnDelay {
    delay: TimeMillis,
    state: RunOnDelayState,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) enum RunOnDelayState {
    Demand,
    RunOn { end: TimeMillis },
    Idle,
}
