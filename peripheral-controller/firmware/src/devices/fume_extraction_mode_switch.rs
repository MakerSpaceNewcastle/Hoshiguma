use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    FumeExtractionModeSwitchResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

type FumeExtractionModeSwitch =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, FumeExtractionMode>;

impl From<FumeExtractionModeSwitchResources> for FumeExtractionModeSwitch {
    fn from(r: FumeExtractionModeSwitchResources) -> Self {
        let input = Input::new(r.switch, Pull::Down);
        Self::new([input])
    }
}

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

impl From<&FumeExtractionMode>
    for hoshiguma_telemetry_protocol::payload::observation::FumeExtractionMode
{
    fn from(value: &FumeExtractionMode) -> Self {
        match value {
            FumeExtractionMode::Automatic => Self::Automatic,
            FumeExtractionMode::OverrideRun => Self::OverrideRun,
        }
    }
}

impl StateFromDigitalInputs<1> for FumeExtractionMode {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Automatic,
            Level::High => Self::OverrideRun,
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
    let mut input: FumeExtractionModeSwitch = r.into();

    let mut ticker = Ticker::every(Duration::from_millis(250));

    let tx = FUME_EXTRACTION_MODE_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = input.update() {
            queue_telemetry_message(Payload::Observation(
                ObservationPayload::FumeExtractionMode((&state).into()),
            ))
            .await;

            tx.send(state);
        }
    }
}
