use crate::{
    hal::TimeMillis,
    io::inputs::{ExtractionMode, Inputs},
};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum ExtractionStatus {
    Idle,
    Demand,
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
    30_000
};

impl super::StatusUpdate for ExtractionStatus {
    fn update(&self, time: TimeMillis, current: &Inputs) -> Self {
        if current.extraction_mode == ExtractionMode::Run || current.machine_running {
            Self::Demand
        } else {
            match &self {
                Self::Idle => Self::Idle,
                Self::Demand => Self::RunOn {
                    demand_end_time: time,
                },
                Self::RunOn { demand_end_time } => {
                    if time - demand_end_time > EXTRACTOR_RUN_ON_DELAY {
                        Self::Idle
                    } else {
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
