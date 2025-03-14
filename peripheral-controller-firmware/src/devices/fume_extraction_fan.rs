use crate::{telemetry::queue_telemetry_message, FumeExtractionFanResources};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{
    control::{ControlPayload, FumeExtractionFan},
    Payload,
};

pub(crate) static FUME_EXTRACTION_FAN: Watch<CriticalSectionRawMutex, FumeExtractionFan, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionFanResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = FUME_EXTRACTION_FAN.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::FumeExtractionFan(
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
