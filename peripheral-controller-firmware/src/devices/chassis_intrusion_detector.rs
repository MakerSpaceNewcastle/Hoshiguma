use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_event, ChassisIntrusionDetectResources,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::peripheral_controller::{
    event::{EventKind, ObservationEvent},
    types::ChassisIntrusion,
};

pub(crate) static CHASSIS_INTRUSION_CHANGED: Watch<CriticalSectionRawMutex, ChassisIntrusion, 1> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: ChassisIntrusionDetectResources) {
    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_micros(50));

    let tx = CHASSIS_INTRUSION_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => ChassisIntrusion::Intruded,
            Level::High => ChassisIntrusion::Normal,
        };

        queue_telemetry_event(EventKind::Observation(ObservationEvent::ChassisIntrusion(
            state.clone(),
        )))
        .await;

        tx.send(state);
    }
}
