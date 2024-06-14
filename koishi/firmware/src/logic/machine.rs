use super::StatusUpdate;
use crate::hal::TimeMillis;
use enumset::EnumSet;
use hoshiguma_foundational_data::koishi::{Inputs, MachineProblem, MachineStatus};

impl StatusUpdate for MachineStatus {
    fn update(&self, _: TimeMillis, current: &Inputs) -> Self {
        // Assume the machine is idle
        let mut state = Self::Idle;

        // Check for fault conditions
        let mut problems = EnumSet::new();

        if !current.doors_closed {
            problems.insert(MachineProblem::DoorOpen);
        }

        if !current.external_enable {
            problems.insert(MachineProblem::External);
        }

        if !problems.is_empty() {
            state = Self::Problem(problems);
        }

        // Check for running condition
        if state == Self::Idle && current.machine_running {
            state = Self::Running;
        }

        state
    }
}
