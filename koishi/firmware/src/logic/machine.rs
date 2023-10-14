use super::StatusUpdate;
use crate::{hal::TimeMillis, io::inputs::Inputs};
use enumset::{EnumSet, EnumSetType};
use serde::Serialize;
use ufmt::derive::uDebug;

#[derive(Clone, PartialEq, Serialize)]
pub(crate) enum MachineStatus {
    /// The machine is currently running a job.
    Running,

    /// The machine is not running, but is ready to run a job.
    Idle,

    /// The machine is not running, and cannot run for some reason.
    Problem(EnumSet<MachineProblem>),
}

impl Default for MachineStatus {
    fn default() -> Self {
        Self::Idle
    }
}

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

#[derive(uDebug, Serialize, EnumSetType)]
pub(crate) enum MachineProblem {
    /// Any door to a protected area is open.
    DoorOpen,

    /// An external controller has indicated a fault condition.
    External,
}
