pub(crate) mod air_assist;
pub(crate) mod extraction;
pub(crate) mod machine;
mod run_on_delay;

use crate::hal::TimeMillis;
use hoshiguma_foundational_data::koishi::Inputs;

pub(crate) trait StatusUpdate {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self;
}
