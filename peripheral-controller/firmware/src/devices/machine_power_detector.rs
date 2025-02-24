use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    MachinePowerDetectResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker, Timer};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

type MachinePowerDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, MachinePower>;

impl From<MachinePowerDetectResources> for MachinePowerDetector {
    fn from(r: MachinePowerDetectResources) -> Self {
        let input = Input::new(r.detect, Pull::Down);
        Self::new([input])
    }
}

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

impl StateFromDigitalInputs<1> for MachinePower {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Off,
            Level::High => Self::On,
        }
    }
}

pub(crate) static MACHINE_POWER_CHANGED: Watch<CriticalSectionRawMutex, MachinePower, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachinePowerDetectResources) {
    let mut machine_power_detector: MachinePowerDetector = r.into();

    let mut ticker = Ticker::every(Duration::from_millis(100));

    let tx = MACHINE_POWER_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = machine_power_detector.update() {
            queue_telemetry_message(Payload::Observation(ObservationPayload::MachinePower(
                (&state).into(),
            )))
            .await;

            if state == MachinePower::On {
                // Wait a while before sending state, allows 24V bus to stabalise and
                // controller to boot.
                let delay = Duration::from_secs(1);
                ticker.reset_after(delay);
                Timer::after(delay).await;
            }

            tx.send(state);
        }
    }
}
