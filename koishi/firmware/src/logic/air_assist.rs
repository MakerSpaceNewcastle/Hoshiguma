use crate::{hal::TimeMillis, logic::run_on_delay::RunOnDelayExt};
use hoshiguma_foundational_data::koishi::{run_on_delay::RunOnDelay, AirAssistStatus, Inputs};

const AIR_ASSIST_RUN_ON_DELAY: TimeMillis = 500;

pub(crate) trait AirAssistStatusExt {
    fn default() -> Self;
    fn active(&self) -> bool;
}

impl AirAssistStatusExt for AirAssistStatus {
    fn default() -> Self {
        Self {
            state: RunOnDelay::new(AIR_ASSIST_RUN_ON_DELAY),
        }
    }

    fn active(&self) -> bool {
        self.state.should_run()
    }
}

impl super::StatusUpdate for AirAssistStatus {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self {
        let mut new_state = self.clone();

        let demand = current.air_pump_demand;
        new_state.state.update(time, demand);

        new_state
    }
}
