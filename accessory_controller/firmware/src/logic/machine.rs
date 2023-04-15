use crate::{hal::TimeMillis, io::inputs::Inputs};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum MachineStatus {
    Running { air_pump: bool },
    Idle,
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
    DoorOpen,
    CoolingFault,
}
