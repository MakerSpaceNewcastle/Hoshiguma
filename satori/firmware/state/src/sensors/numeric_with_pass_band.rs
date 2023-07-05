use super::{
    numeric_common::{lower_threshold, upper_threshold},
    CalculateState, State,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NumericWithPassBand<T>
where
    T: PartialOrd,
{
    value: T,

    low_alarm: Option<T>,
    low_warn: Option<T>,

    high_warn: Option<T>,
    high_alarm: Option<T>,
}

impl<T: std::cmp::PartialOrd> CalculateState for NumericWithPassBand<T> {
    fn state(&self) -> State {
        let res = lower_threshold(&self.value, &self.low_warn, &self.low_warn);
        if res != State::Normal {
            return res;
        }

        let res = upper_threshold(&self.value, &self.high_warn, &self.high_alarm);
        if res != State::Normal {
            return res;
        }

        State::Normal
    }
}
