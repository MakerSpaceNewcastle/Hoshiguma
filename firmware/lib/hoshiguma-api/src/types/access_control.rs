use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessControlRawInput {
    Idle,
    Denied,
    Granted,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
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
