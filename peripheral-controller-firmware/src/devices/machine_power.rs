use crate::{telemetry::queue_telemetry_event, MachinePowerResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind},
    types::MachinePower,
};
use pico_plc_bsp::embassy_rp::gpio::{Level, Output};

pub(crate) struct MachinePowerOutput {
    relay: Output<'static>,
}

impl MachinePowerOutput {
    pub(crate) fn new(r: MachinePowerResources) -> Self {
        let relay = Output::new(r.relay, Level::Low);
        Self { relay }
    }

    pub(crate) fn set(&mut self, setting: MachinePower) {
        let level = match setting {
            MachinePower::Off => Level::Low,
            MachinePower::On => Level::High,
        };
        self.relay.set_level(level);
    }
}

pub(crate) static MACHINE_POWER: Watch<CriticalSectionRawMutex, MachinePower, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachinePowerResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("mach pwr o/p").await;

    let mut output = MachinePowerOutput::new(r);
    let mut rx = MACHINE_POWER.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_event(EventKind::Control(ControlEvent::MachinePower(
            setting.clone(),
        )))
        .await;

        // Set relay output
        output.set(setting);
    }
}
