use crate::{hal::TimeMillis, logic::run_on_delay::RunOnDelayExt};
use hoshiguma_foundational_data::koishi::{
    run_on_delay::RunOnDelay, ExtractionMode, ExtractionStatus, Inputs,
};

/// Time in milliseconds that the extractor will continue to run after demand has ceased.
const EXTRACTOR_RUN_ON_DELAY: TimeMillis = if cfg!(feature = "simulator") {
    500
} else {
    45_000
};

pub(crate) trait ExtractionStatusExt {
    fn default() -> Self;
    fn active(&self) -> bool;
}

impl ExtractionStatusExt for ExtractionStatus {
    fn default() -> Self {
        Self {
            state: RunOnDelay::new(EXTRACTOR_RUN_ON_DELAY),
            r#override: false,
        }
    }

    fn active(&self) -> bool {
        self.state.should_run() || self.r#override
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
