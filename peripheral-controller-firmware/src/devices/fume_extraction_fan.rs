use crate::{telemetry::queue_telemetry_event, FumeExtractionFanResources};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind},
    types::FumeExtractionFan,
};

pub(crate) static FUME_EXTRACTION_FAN: Watch<CriticalSectionRawMutex, FumeExtractionFan, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionFanResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("fe fan o").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = FUME_EXTRACTION_FAN.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_event(EventKind::Control(ControlEvent::FumeExtractionFan(
            setting.clone(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            FumeExtractionFan::Idle => Level::Low,
            FumeExtractionFan::Run => Level::High,
        };
        output.set_level(level);
    }
}
