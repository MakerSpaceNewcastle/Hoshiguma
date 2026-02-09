use crate::{
    LaserEnableResources, logic::interlock::interlock_action_rx,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Level, Output};
use hoshiguma_api::{InterlockAction, LaserEnable};
use hoshiguma_common::telemetry::format_influx_line;

pub(crate) struct LaserEnableOutput {
    relay: Output<'static>,
}

impl LaserEnableOutput {
    pub(crate) fn new(r: LaserEnableResources) -> Self {
        let relay = Output::new(r.relay, Level::Low);
        Self { relay }
    }

    pub(crate) fn set(&mut self, setting: LaserEnable) {
        let level = match setting {
            LaserEnable::Inhibit => Level::Low,
            LaserEnable::Enable => Level::High,
        };
        self.relay.set_level(level);
    }

    /// Ensure the laser enable relay is turned off, disabling the laser power supply from turning
    /// on.
    #[cfg(not(feature = "panic-probe"))]
    pub(crate) fn set_safe(&mut self) {
        self.set(LaserEnable::Inhibit);
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: LaserEnableResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("laser enable output").await;

    let mut output = LaserEnableOutput::new(r);
    let mut rx = interlock_action_rx();

    loop {
        let interlock = rx.changed().await;

        let setting = match interlock {
            InterlockAction::Normal => LaserEnable::Enable,
            _ => LaserEnable::Inhibit,
        };

        queue_telemetry_data_point(format_influx_line(
            format_args!("laser_enable value=\"{setting}\""),
            crate::wall_time::now(),
        ));

        output.set(setting);
    }
}
