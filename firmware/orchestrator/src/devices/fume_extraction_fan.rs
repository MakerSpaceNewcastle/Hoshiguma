use crate::{FumeExtractionFanResources, telemetry::queue_telemetry_data_point};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_core::{telemetry::AsTelemetry, types::FumeExtractionFan};

pub(crate) static FUME_EXTRACTION_FAN: Watch<CriticalSectionRawMutex, FumeExtractionFan, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionFanResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("fe fan o/p").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = FUME_EXTRACTION_FAN.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        for dp in setting.telemetry() {
            queue_telemetry_data_point(dp);
        }

        let level = match setting {
            FumeExtractionFan::Idle => Level::Low,
            FumeExtractionFan::Run => Level::High,
        };
        output.set_level(level);
    }
}
