use crate::{
    AirAssistDemandDetectResources, input_change_detector::InputChangeDetector,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_api::AirAssistDemand;
use hoshiguma_common::telemetry::format_influx_line;

crate::variable_watch!(air_assist_demand, AirAssistDemand, 1);

#[embassy_executor::task]
pub(crate) async fn task(r: AirAssistDemandDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("air assist demand detect").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = InputChangeDetector::new(pin);

    let tx = AIR_ASSIST_DEMAND.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => AirAssistDemand::Idle,
            Level::High => AirAssistDemand::Demand,
        };

        queue_telemetry_data_point(format_influx_line(
            format_args!("air_assist_demand value=\"{state}\""),
            crate::wall_time::now(),
        ));

        tx.send(state);
    }
}
