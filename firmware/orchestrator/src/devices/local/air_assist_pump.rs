use crate::{
    AirAssistPumpResources, logic::air_assist::air_assist_pump_rx,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Level, Output};
use hoshiguma_api::AirAssistPump;
use hoshiguma_common::telemetry::format_influx_line;

#[embassy_executor::task]
pub(crate) async fn task(r: AirAssistPumpResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("air assist pump output").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = air_assist_pump_rx();

    loop {
        let setting = rx.changed().await;

        queue_telemetry_data_point(format_influx_line(
            format_args!("air_assist_pump value=\"{setting}\""),
            crate::wall_time::now(),
        ));

        let level = match setting {
            AirAssistPump::Idle => Level::Low,
            AirAssistPump::Run => Level::High,
        };
        output.set_level(level);
    }
}
