use crate::{
    hal::TimeMillis,
    io::inputs::{ExtractionMode, Inputs},
};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum ExtractionStatus {
    /// The extractor is not in demand and is not running.
    Idle,

    /// The extractor is currently in demand and is running.
    Demand,

    /// The extractor is no longer in demand, but will continue to run to cycle in clean air.
    RunOn { demand_end_time: TimeMillis },
}

impl Default for ExtractionStatus {
    fn default() -> Self {
        Self::Idle
    }
}

const EXTRACTOR_RUN_ON_DELAY: TimeMillis = if cfg!(feature = "simulator") {
    500
} else {
    // Time in milliseconds that the extractor will continue to run after demand has ceased.
    15_000
};

impl super::StatusUpdate for ExtractionStatus {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self {
        // If the extraction mode switch is set to override or the machine is running, then the
        // extractor should be running.
        if current.extraction_mode == ExtractionMode::Run || current.machine_running {
            Self::Demand
        } else {
            match &self {
                // If the extractor is not in demand, then it remains not in demand.
                Self::Idle => Self::Idle,
                // If the extractor was in demand, but no longer is, then start the run on delay.
                Self::Demand => Self::RunOn {
                    demand_end_time: time,
                },
                Self::RunOn { demand_end_time } => {
                    if time - demand_end_time > EXTRACTOR_RUN_ON_DELAY {
                        // If the extractor is not in demand, still running, but no longer in the run
                        // on period, then it should not be running.
                        Self::Idle
                    } else {
                        // If the extractor is not in demand, still running, and still in the run
                        // on period, then it should continue to run.
                        Self::RunOn {
                            demand_end_time: *demand_end_time,
                        }
                    }
                }
            }
        }
    }
}

impl ExtractionStatus {
    pub fn fan_active(&self) -> bool {
        match *self {
            Self::Idle => false,
            Self::Demand => true,
            Self::RunOn { demand_end_time: _ } => true,
        }
    }
}
