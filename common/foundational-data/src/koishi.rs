use crate::TimeMillis;
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Payload {
    InputsChanged(Inputs),
    OutputsChanged(Outputs),

    MachineStatusChanged(MachineStatus),
    AirAssistStatusChanged(AirAssistStatus),
    ExtractionStatusChanged(ExtractionStatus),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Inputs {
    pub doors_closed: bool,
    pub external_enable: bool,
    pub machine_running: bool,
    pub air_pump_demand: bool,
    pub extraction_mode: ExtractionMode,
}

impl From<&Inputs> for Payload {
    fn from(value: &Inputs) -> Self {
        Self::InputsChanged(value.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExtractionMode {
    Normal,
    Run,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Outputs {
    pub controller_machine_alarm: AlarmState,
    pub controller_cooling_alarm: AlarmState,
    pub laser_enable: bool,
    pub status_light: StatusLight,
    pub air_pump: bool,
    pub extractor_fan: bool,
}

impl From<&Outputs> for Payload {
    fn from(value: &Outputs) -> Self {
        Self::OutputsChanged(value.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AlarmState {
    Normal,
    Alarm,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StatusLight {
    Green,
    Amber,
    Red,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MachineStatus {
    /// The machine is currently running a job.
    Running,

    /// The machine is not running, but is ready to run a job.
    Idle,

    /// The machine is not running, and cannot run for some reason.
    Problem(EnumSet<MachineProblem>),
}

impl From<&MachineStatus> for Payload {
    fn from(value: &MachineStatus) -> Self {
        Self::MachineStatusChanged(value.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, EnumSetType)]
pub enum MachineProblem {
    /// Any door to a protected area is open.
    DoorOpen,

    /// An external controller has indicated a fault condition.
    External,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AirAssistStatus {
    pub state: run_on_delay::RunOnDelay<TimeMillis>,
}

impl From<&AirAssistStatus> for Payload {
    fn from(value: &AirAssistStatus) -> Self {
        Self::AirAssistStatusChanged(value.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExtractionStatus {
    pub state: run_on_delay::RunOnDelay<TimeMillis>,
    pub r#override: bool,
}

impl From<&ExtractionStatus> for Payload {
    fn from(value: &ExtractionStatus) -> Self {
        Self::ExtractionStatusChanged(value.clone())
    }
}

pub mod run_on_delay {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    pub struct RunOnDelay<T: PartialEq> {
        pub delay: T,
        pub state: State<T>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    pub enum State<T> {
        Demand,
        RunOn { end: T },
        Idle,
    }
}
