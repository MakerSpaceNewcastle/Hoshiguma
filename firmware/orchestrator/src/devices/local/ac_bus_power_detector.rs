use crate::{
    AcBusPowerDetectResources, input_change_detector::InputChangeDetector,
    logic::interlock::update_monitor_severity, telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_time::{Duration, Timer};
use hoshiguma_api::{AcBusPower, Monitor, Severity};
use hoshiguma_common::telemetry::format_influx_line;

crate::variable_watch!(ac_bus_power, AcBusPower, 4);

#[embassy_executor::task]
pub(crate) async fn task(r: AcBusPowerDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("ac bus power detect").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = InputChangeDetector::new(pin);

    let tx = AC_BUS_POWER.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => AcBusPower::Off,
            Level::High => AcBusPower::On,
        };

        update_monitor_severity(
            Monitor::AcBusPower,
            match state {
                AcBusPower::On => Severity::Normal,
                AcBusPower::Off => Severity::Critical,
            },
        )
        .await;

        queue_telemetry_data_point(format_influx_line(
            format_args!("ac_bus_power value=\"{state}\""),
            crate::wall_time::now(),
        ));

        if state == AcBusPower::On {
            // Wait a while before sending state, allows 24V bus to stabalise and
            // controller to boot.
            Timer::after(Duration::from_secs(1)).await;
        }

        tx.send(state);
    }
}
