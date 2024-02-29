use super::{numeric_common::upper_threshold, CalculateState, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NumericWithUpperThreshold<T>
where
    T: PartialOrd,
{
    value: T,

    high_warn: Option<T>,
    high_alarm: Option<T>,
}

impl<T: std::cmp::PartialOrd> CalculateState for NumericWithUpperThreshold<T> {
    fn state(&self) -> State {
        upper_threshold(&self.value, &self.high_warn, &self.high_alarm)
    }
}
