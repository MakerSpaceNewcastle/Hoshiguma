use crate::{hal::TimeMillis, io::inputs::Inputs, logic::run_on_delay::RunOnDelay};
use ufmt::derive::uDebug;

const AIR_ASSIST_RUN_ON_DELAY: TimeMillis = 500;

#[derive(uDebug, Clone, PartialEq)]
pub(crate) struct AirAssistStatus {
    state: RunOnDelay<TimeMillis>,
}

impl Default for AirAssistStatus {
    fn default() -> Self {
        Self {
            state: RunOnDelay::new(AIR_ASSIST_RUN_ON_DELAY),
        }
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

impl AirAssistStatus {
    pub fn active(&self) -> bool {
        self.state.should_run()
    }
}
