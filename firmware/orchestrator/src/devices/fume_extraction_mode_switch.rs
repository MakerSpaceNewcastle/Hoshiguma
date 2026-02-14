use crate::{
    FumeExtractionModeSwitchResources, polled_input::PolledInput,
    telemetry::queue_telemetry_data_point,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_core::{telemetry::AsTelemetry, types::FumeExtractionMode};
use pico_plc_bsp::embassy_rp::gpio::{Input, Level, Pull};

pub(crate) static FUME_EXTRACTION_MODE_CHANGED: Watch<
    CriticalSectionRawMutex,
    FumeExtractionMode,
    2,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionModeSwitchResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("fe mode sw").await;

    let pin = Input::new(r.switch, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(50));

    let tx = FUME_EXTRACTION_MODE_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => FumeExtractionMode::Automatic,
            Level::High => FumeExtractionMode::OverrideRun,
        };

        for dp in state.telemetry() {
            queue_telemetry_data_point(dp);
        }

        tx.send(state);
    }
}
