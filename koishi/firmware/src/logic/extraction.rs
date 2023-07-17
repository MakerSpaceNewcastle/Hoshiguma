use crate::{
    hal::TimeMillis,
    io::inputs::{ExtractionMode, Inputs},
    logic::run_on_delay::RunOnDelay,
};
use serde::Serialize;
use ufmt::derive::uDebug;

const EXTRACTOR_RUN_ON_DELAY: TimeMillis = if cfg!(feature = "simulator") {
    500
} else {
    // Time in milliseconds that the extractor will continue to run after demand has ceased.
    45_000
};

#[derive(uDebug, Clone, PartialEq, Serialize)]
pub(crate) struct ExtractionStatus {
    state: RunOnDelay<TimeMillis>,
    r#override: bool,
}

impl Default for ExtractionStatus {
    fn default() -> Self {
        Self {
            state: RunOnDelay::new(EXTRACTOR_RUN_ON_DELAY),
            r#override: false,
        }
    }
}

impl super::StatusUpdate for ExtractionStatus {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self {
        let mut new_state = self.clone();

        new_state.state.update(time, current.machine_running);
        new_state.r#override = current.extraction_mode == ExtractionMode::Run;

        new_state
    }
}

impl ExtractionStatus {
    pub fn active(&self) -> bool {
        self.state.should_run() || self.r#override
    }
}
