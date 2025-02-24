use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    MachineRunDetectResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

type MachineRunDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, MachineRunStatus>;

impl From<MachineRunDetectResources> for MachineRunDetector {
    fn from(r: MachineRunDetectResources) -> Self {
        let input = Input::new(r.detect, Pull::Down);
        Self::new([input])
    }
}

#[derive(Clone, Format)]
pub(crate) enum MachineRunStatus {
    Idle,
    Running,
}

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

pub(crate) static MACHINE_RUNNING_CHANGED: Watch<CriticalSectionRawMutex, MachineRunStatus, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineRunDetectResources) {
    let mut input: MachineRunDetector = r.into();

    let mut ticker = Ticker::every(Duration::from_millis(50));

    let tx = MACHINE_RUNNING_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = input.update() {
            queue_telemetry_message(Payload::Observation(ObservationPayload::MachineRun(
                (&state).into(),
            )))
            .await;

            tx.send(state);
        }
    }
}
