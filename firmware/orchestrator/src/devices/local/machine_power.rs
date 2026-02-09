use crate::{
    MachinePowerResources, logic::machine_power::machine_power_rx,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Level, Output};
use hoshiguma_api::DesiredMachinePower;
use hoshiguma_common::telemetry::format_influx_line;

#[embassy_executor::task]
pub(crate) async fn task(r: MachinePowerResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine power output").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = machine_power_rx();

    loop {
        let setting = rx.changed().await;

        queue_telemetry_data_point(format_influx_line(
            format_args!("machine_power value=\"{setting}\""),
            crate::wall_time::now(),
        ));

        let level = match setting {
            DesiredMachinePower::Off => Level::Low,
            DesiredMachinePower::On => Level::High,
        };
        output.set_level(level);
    }
}
