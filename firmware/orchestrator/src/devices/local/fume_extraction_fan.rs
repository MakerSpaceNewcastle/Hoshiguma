use crate::{
    FumeExtractionFanResources, logic::fume_extraction::fume_extraction_fan_rx,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Level, Output};
use hoshiguma_api::FumeExtractionFan;
use hoshiguma_common::telemetry::format_influx_line;

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionFanResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("extraction fan output").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = fume_extraction_fan_rx();

    loop {
        let setting = rx.changed().await;

        queue_telemetry_data_point(format_influx_line(
            format_args!("fume_extraction_fan value=\"{setting}\""),
            crate::wall_time::now(),
        ));

        let level = match setting {
            FumeExtractionFan::Idle => Level::Low,
            FumeExtractionFan::Run => Level::High,
        };
        output.set_level(level);
    }
}
