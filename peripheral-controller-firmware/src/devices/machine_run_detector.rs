use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_event, MachineRunDetectResources,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::peripheral_controller::{
    event::{EventKind, ObservationEvent},
    types::MachineRun,
};

pub(crate) static MACHINE_RUNNING_CHANGED: Watch<CriticalSectionRawMutex, MachineRun, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineRunDetectResources) {
    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = MACHINE_RUNNING_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachineRun::Idle,
            Level::High => MachineRun::Running,
        };

        queue_telemetry_event(EventKind::Observation(ObservationEvent::MachineRun(
            state.clone(),
        )))
        .await;

        tx.send(state);
    }
}
