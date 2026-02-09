use crate::{
    DoorsDetectResources, input_change_detector::InputChangeDetector,
    logic::interlock::update_monitor_severity, telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_api::{Doors, Monitor, Severity};
use hoshiguma_common::telemetry::format_influx_line;

#[embassy_executor::task]
pub(crate) async fn task(r: DoorsDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("doors detect").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = InputChangeDetector::new(pin);

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => Doors::Open,
            Level::High => Doors::Closed,
        };

        update_monitor_severity(
            Monitor::Doors,
            match state {
                Doors::Closed => Severity::Normal,
                Doors::Open => Severity::Critical,
            },
        )
        .await;

        queue_telemetry_data_point(format_influx_line(
            format_args!("doors value=\"{state}\""),
            crate::wall_time::now(),
        ));
    }
}
