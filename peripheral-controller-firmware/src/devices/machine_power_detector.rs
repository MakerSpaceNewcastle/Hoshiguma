use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_event, MachinePowerDetectResources,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Timer};
use hoshiguma_protocol::peripheral_controller::{
    event::{EventKind, ObservationEvent},
    types::MachinePower,
};

pub(crate) static MACHINE_POWER_CHANGED: Watch<CriticalSectionRawMutex, MachinePower, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachinePowerDetectResources) {
    crate::trace::name_task("mach pwr det").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = MACHINE_POWER_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachinePower::Off,
            Level::High => MachinePower::On,
        };

        queue_telemetry_event(EventKind::Observation(ObservationEvent::MachinePower(
            state.clone(),
        )))
        .await;

        if state == MachinePower::On {
            // Wait a while before sending state, allows 24V bus to stabalise and
            // controller to boot.
            Timer::after(Duration::from_secs(1)).await;
        }

        tx.send(state);
    }
}
