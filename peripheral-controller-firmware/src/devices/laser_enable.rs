use crate::{LaserEnableResources, telemetry::queue_telemetry_data_point};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_core::{telemetry::AsTelemetry, types::LaserEnable};
use pico_plc_bsp::embassy_rp::gpio::{Level, Output};

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
    pub(crate) fn set_panic(&mut self) {
        self.set(LaserEnable::Inhibit);
    }
}

pub(crate) static LASER_ENABLE: Watch<CriticalSectionRawMutex, LaserEnable, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: LaserEnableResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("laser en o/p").await;

    let mut output = LaserEnableOutput::new(r);
    let mut rx = LASER_ENABLE.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        for dp in setting.telemetry() {
            queue_telemetry_data_point(dp);
        }

        output.set(setting);
    }
}
