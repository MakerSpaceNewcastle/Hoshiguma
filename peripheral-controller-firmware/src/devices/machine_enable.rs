use crate::{MachineEnableResources, telemetry::queue_telemetry_data_point};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_core::{telemetry::AsTelemetry, types::MachineEnable};
use pico_plc_bsp::embassy_rp::gpio::{Level, Output};

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
    pub(crate) fn set_panic(&mut self) {
        self.set(MachineEnable::Inhibit);
    }
}

pub(crate) static MACHINE_ENABLE: Watch<CriticalSectionRawMutex, MachineEnable, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("mach en o/p").await;

    let mut output = MachineEnableOutput::new(r);
    let mut rx = MACHINE_ENABLE.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        for dp in setting.telemetry() {
            queue_telemetry_data_point(dp);
        }

        output.set(setting);
    }
}
