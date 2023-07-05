use super::{CalculateState, SensorReading, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum CoolantLevel {
    Full,
    Acceptable,
    Low,
}

impl Default for CoolantLevel {
    fn default() -> Self {
        CoolantLevel::Low
    }
}

impl CalculateState for CoolantLevel {
    fn state(&self) -> State {
        match self {
            Self::Full => State::Normal,
            Self::Acceptable => State::Normal,
            Self::Low => State::Warn,
        }
    }
}

pub type CoolantLevelReading = SensorReading<CoolantLevel>;
