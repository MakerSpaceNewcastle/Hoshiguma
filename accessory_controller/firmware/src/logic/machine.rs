use crate::{hal::TimeMillis, io::inputs::Inputs};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum MachineStatus {
    /// The machine is currently running a job.
    Running { air_pump: bool },

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
            Self::Running {
                air_pump: current.air_pump_demand,
            }
        } else {
            Self::Idle
        }
    }
}

#[derive(uDebug, PartialEq)]
pub(crate) enum MachineProblem {
    /// Any door to a protected area is open.
    DoorOpen,

    /// The laser tube cooling system has failed.
    CoolingFault,
}
