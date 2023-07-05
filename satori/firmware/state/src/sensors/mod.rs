mod coolant_level;
mod numeric_common;
mod numeric_with_pass_band;
mod numeric_with_upper_threshold;

pub use self::{
    coolant_level::{CoolantLevel, CoolantLevelReading},
    numeric_with_pass_band::NumericWithPassBand,
    numeric_with_upper_threshold::NumericWithUpperThreshold,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum State {
    Normal,
    Warn,
    Critical,
}

impl Default for State {
    fn default() -> Self {
        Self::Critical
    }
}

pub trait CalculateState {
    fn state(&self) -> State;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SensorValue<T> {
    value: T,
    // state: State,
}

impl<T> SensorValue<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }
}

impl<T: num::Num> CalculateState for SensorValue<T> {
    fn state(&self) -> State {
        State::Normal
    }
}

impl CalculateState for SensorValue<CoolantLevel> {
    fn state(&self) -> State {
        self.value.state()
    }
}

impl<T: PartialOrd> CalculateState for SensorValue<NumericWithPassBand<T>> {
    fn state(&self) -> State {
        self.value.state()
    }
}

impl<T: PartialOrd> CalculateState for SensorValue<NumericWithUpperThreshold<T>> {
    fn state(&self) -> State {
        self.value.state()
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SensorError {
    #[error("Sensor read failed")]
    ReadFailed,
}

pub type SensorReading<T> = Result<SensorValue<T>, SensorError>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn state_generic_u32() {
        let v: SensorValue<u32> = SensorValue::default();
        assert_eq!(v.state(), State::Normal);
    }

    #[test]
    fn state_coolant_level() {
        let mut v = SensorValue::default();
        assert_eq!(v.value, CoolantLevel::Low);
        assert_eq!(v.state(), State::Warn);

        v.set(CoolantLevel::Full);
        assert_eq!(v.state(), State::Normal);

        v.set(CoolantLevel::Low);
        assert_eq!(v.state(), State::Warn);
    }
}
