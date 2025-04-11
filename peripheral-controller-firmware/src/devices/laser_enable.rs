use crate::{telemetry::queue_telemetry_event, LaserEnableResources};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind},
    types::LaserEnable,
};

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
    crate::trace::name_task("las en o").await;

    let mut output = LaserEnableOutput::new(r);
    let mut rx = LASER_ENABLE.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_event(EventKind::Control(ControlEvent::LaserEnable(
            setting.clone(),
        )))
        .await;

        // Set relay output
        output.set(setting);
    }
}
