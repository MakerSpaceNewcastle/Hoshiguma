use crate::{hal::TimeMillis, io::inputs::Inputs};
use serde::Serialize;
use ufmt::derive::uDebug;

#[derive(Clone, uDebug, PartialEq, Serialize)]
pub(crate) enum MachineStatus {
    /// The machine is currently running a job.
    Running,

    /// The machine is not running, but is ready to run a job.
    Idle,

    /// The machine is not running, and cannot run for some reason.
    Problem(MachineProblem),
}

impl Default for MachineStatus {
    fn default() -> Self {
        Self::Idle
    }
}

impl super::StatusUpdate for MachineStatus {
    fn update(&self, _: TimeMillis, current: &Inputs) -> Self {
        if !current.doors_closed {
            Self::Problem(MachineProblem::DoorOpen)
        } else if !current.cooling_ok {
            Self::Problem(MachineProblem::CoolingFault)
        } else if current.machine_running {
            Self::Running
        } else {
            Self::Idle
        }
    }
}

#[derive(Clone, uDebug, PartialEq, Serialize)]
pub(crate) enum MachineProblem {
    /// Any door to a protected area is open.
    DoorOpen,

    /// The laser tube cooling system has failed.
    CoolingFault,
}
