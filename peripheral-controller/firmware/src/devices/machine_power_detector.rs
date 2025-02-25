use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_message, MachinePowerDetectResources,
};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Timer};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) enum MachinePower {
    Off,
    On,
}

impl From<&MachinePower> for hoshiguma_telemetry_protocol::payload::observation::MachinePower {
    fn from(value: &MachinePower) -> Self {
        match value {
            MachinePower::Off => Self::Off,
            MachinePower::On => Self::On,
        }
    }
}

pub(crate) static MACHINE_POWER_CHANGED: Watch<CriticalSectionRawMutex, MachinePower, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachinePowerDetectResources) {
    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = MACHINE_POWER_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachinePower::Off,
            Level::High => MachinePower::On,
        };

        queue_telemetry_message(Payload::Observation(ObservationPayload::MachinePower(
            (&state).into(),
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
