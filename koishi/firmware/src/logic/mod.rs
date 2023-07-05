pub(crate) mod air_assist;
pub(crate) mod extraction;
pub(crate) mod machine;
mod run_on_delay;

use crate::{hal::TimeMillis, io::inputs::Inputs};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum AlarmState {
    Normal,
    Alarm,
}

#[derive(uDebug, PartialEq)]
pub(crate) enum StatusLight {
    Green,
    Amber,
    Red,
}

pub(crate) trait StatusUpdate {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self;
}
