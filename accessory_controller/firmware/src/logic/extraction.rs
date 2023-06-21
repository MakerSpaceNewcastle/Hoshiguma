use crate::{
    hal::TimeMillis,
    io::inputs::{ExtractionMode, Inputs},
    logic::run_on_delay::RunOnDelay,
};
use ufmt::derive::uDebug;

const EXTRACTOR_RUN_ON_DELAY: TimeMillis = if cfg!(feature = "simulator") {
    500
} else {
    // Time in milliseconds that the extractor will continue to run after demand has ceased.
    45_000
};

#[derive(uDebug, Clone, PartialEq)]
pub(crate) struct ExtractionStatus {
    state: RunOnDelay<TimeMillis>,
}

impl Default for ExtractionStatus {
    fn default() -> Self {
        Self {
            state: RunOnDelay::new(EXTRACTOR_RUN_ON_DELAY),
        }
    }
}

impl super::StatusUpdate for ExtractionStatus {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self {
        let mut new_state = self.clone();

        let demand = current.extraction_mode == ExtractionMode::Run || current.machine_running;
        new_state.state.update(time, demand);

        new_state
    }
}

impl ExtractionStatus {
    pub fn active(&self) -> bool {
        self.state.should_run()
    }
}
