use crate::{telemetry::queue_telemetry_message, MachineEnableResources};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{
    control::{ControlPayload, MachineEnable},
    Payload,
};

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
    pub(crate) fn set_panic(&mut self) {
        self.set(MachineEnable::Inhibit);
    }
}

pub(crate) static MACHINE_ENABLE: Watch<CriticalSectionRawMutex, MachineEnable, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    let mut output = MachineEnableOutput::new(r);
    let mut rx = MACHINE_ENABLE.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::MachineEnable(
            setting.clone(),
        )))
        .await;

        // Set relay output
        output.set(setting);
    }
}
