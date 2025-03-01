use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_message,
    FumeExtractionModeSwitchResources,
};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::payload::{observation::ObservationPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

impl From<&FumeExtractionMode> for hoshiguma_protocol::payload::observation::FumeExtractionMode {
    fn from(value: &FumeExtractionMode) -> Self {
        match value {
            FumeExtractionMode::Automatic => Self::Automatic,
            FumeExtractionMode::OverrideRun => Self::OverrideRun,
        }
    }
}

pub(crate) static FUME_EXTRACTION_MODE_CHANGED: Watch<
    CriticalSectionRawMutex,
    FumeExtractionMode,
    2,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionModeSwitchResources) {
    let pin = Input::new(r.switch, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(50));

    let tx = FUME_EXTRACTION_MODE_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => FumeExtractionMode::Automatic,
            Level::High => FumeExtractionMode::OverrideRun,
        };

        queue_telemetry_message(Payload::Observation(
            ObservationPayload::FumeExtractionMode((&state).into()),
        ))
        .await;

        tx.send(state);
    }
}
