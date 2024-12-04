use crate::io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static MACHINE_RUNNING_CHANGED: Watch<CriticalSectionRawMutex, MachineRunStatus, 4> =
    Watch::new();

#[macro_export]
macro_rules! init_machine_run_detector {
    ($p:expr) => {{
        // Isolated input 3
        let input = embassy_rp::gpio::Input::new($p.PIN_12, embassy_rp::gpio::Pull::Down);

        $crate::devices::machine_run_detector::MachineRunDetector::new([input])
    }};
}

pub(crate) type MachineRunDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, MachineRunStatus>;

#[derive(Clone, Format)]
pub(crate) enum MachineRunStatus {
    Idle,
    Running,
}

#[cfg(feature = "telemetry")]
impl From<&MachineRunStatus>
    for hoshiguma_telemetry_protocol::payload::observation::MachineRunStatus
{
    fn from(value: &MachineRunStatus) -> Self {
        match value {
            MachineRunStatus::Idle => Self::Idle,
            MachineRunStatus::Running => Self::Running,
        }
    }
}

impl StateFromDigitalInputs<1> for MachineRunStatus {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Idle,
            Level::High => Self::Running,
        }
    }
}
