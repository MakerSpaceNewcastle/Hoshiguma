use crate::{DesiredMachinePower, Interlock, MachineRun, Severity};
use defmt::Format;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum AccessControlRawInput {
    Idle,
    Denied,
    Granted,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum AccessControlState {
    Denied,
    Granted,
}

impl From<AccessControlRawInput> for AccessControlState {
    fn from(raw: AccessControlRawInput) -> Self {
        match raw {
            AccessControlRawInput::Idle => AccessControlState::Denied,
            AccessControlRawInput::Denied => AccessControlState::Denied,
            AccessControlRawInput::Granted => AccessControlState::Granted,
        }
    }
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BacklightMode {
    AlwaysOn,
    Auto,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Status,
    HmiInfo,
}

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusScreenInfo {
    pub access_control: AccessControlRawInput,
    pub machine_power: DesiredMachinePower,
    pub interlock: Interlock,
    pub running: MachineRun,
    pub messages: Vec<OnscreenMessage, 8>,
}

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnscreenMessage {
    pub text: String<24>,
    pub severity: Severity,
}
