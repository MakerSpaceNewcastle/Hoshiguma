use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_event, AirAssistDemandDetectResources,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::peripheral_controller::{
    event::{EventKind, ObservationEvent},
    types::AirAssistDemand,
};

pub(crate) static AIR_ASSIST_DEMAND_CHANGED: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: AirAssistDemandDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("aa dmnd det").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = AIR_ASSIST_DEMAND_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => AirAssistDemand::Idle,
            Level::High => AirAssistDemand::Demand,
        };

        queue_telemetry_event(EventKind::Observation(ObservationEvent::AirAssistDemand(
            state.clone(),
        )))
        .await;

        tx.send(state);
    }
}
