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
