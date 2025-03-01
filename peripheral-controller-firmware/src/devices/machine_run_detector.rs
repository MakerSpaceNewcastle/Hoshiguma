use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_message, MachineRunDetectResources,
};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::payload::{observation::ObservationPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum MachineRunStatus {
    Idle,
    Running,
}

impl From<&MachineRunStatus> for hoshiguma_protocol::payload::observation::MachineRunStatus {
    fn from(value: &MachineRunStatus) -> Self {
        match value {
            MachineRunStatus::Idle => Self::Idle,
            MachineRunStatus::Running => Self::Running,
        }
    }
}

pub(crate) static MACHINE_RUNNING_CHANGED: Watch<CriticalSectionRawMutex, MachineRunStatus, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineRunDetectResources) {
    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = MACHINE_RUNNING_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachineRunStatus::Idle,
            Level::High => MachineRunStatus::Running,
        };

        queue_telemetry_message(Payload::Observation(ObservationPayload::MachineRun(
            (&state).into(),
        )))
        .await;

        tx.send(state);
    }
}
