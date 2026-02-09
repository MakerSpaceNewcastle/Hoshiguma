use crate::{
    MachineEnableResources, logic::interlock::interlock_action_rx,
    telemetry::queue_telemetry_data_point,
};
use embassy_rp::gpio::{Level, Output};
use hoshiguma_api::{InterlockAction, MachineEnable};
use hoshiguma_common::telemetry::format_influx_line;

pub(crate) struct MachineEnableOutput {
    relay: Output<'static>,
}

impl MachineEnableOutput {
    pub(crate) fn new(r: MachineEnableResources) -> Self {
        let relay = Output::new(r.relay, Level::Low);
        Self { relay }
    }

    pub(crate) fn set(&mut self, setting: MachineEnable) {
        let level = match setting {
            MachineEnable::Inhibit => Level::Low,
            MachineEnable::Enable => Level::High,
        };
        self.relay.set_level(level);
    }

    /// Ensure the machine enable relay is turned off, disabling the Ruida controller from
    /// attempting to operate the machine.
    #[cfg(not(feature = "panic-probe"))]
    pub(crate) fn set_safe(&mut self) {
        self.set(MachineEnable::Inhibit);
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine enable output").await;

    let mut output = MachineEnableOutput::new(r);
    let mut rx = interlock_action_rx();

    loop {
        let interlock = rx.changed().await;

        let setting = match interlock {
            InterlockAction::Normal => MachineEnable::Enable,
            _ => MachineEnable::Inhibit,
        };

        queue_telemetry_data_point(format_influx_line(
            format_args!("machine_enable value=\"{setting}\""),
            crate::wall_time::now(),
        ));

        output.set(setting);
    }
}
