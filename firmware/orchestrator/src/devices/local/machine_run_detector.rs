use crate::{
    MachineRunDetectResources, input_change_detector::InputChangeDetector,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_api::MachineRun;
use hoshiguma_common::telemetry::format_influx_line;

crate::variable_watch!(machine_run, MachineRun, 4);

#[embassy_executor::task]
pub(crate) async fn task(r: MachineRunDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine run detect").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = InputChangeDetector::new(pin);

    let tx = MACHINE_RUN.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachineRun::Idle,
            Level::High => MachineRun::Running,
        };

        queue_telemetry_data_point(format_influx_line(
            format_args!("machine_run value=\"{state}\""),
            crate::wall_time::now(),
        ));

        tx.send(state);
    }
}
