use crate::{telemetry::queue_telemetry_message, MachineEnableResources};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{
    control::{ControlPayload, MachineEnable},
    Payload,
};

pub(crate) static MACHINE_ENABLE: Watch<CriticalSectionRawMutex, MachineEnable, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    let mut output = Output::new(r.relay, Level::Low);
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
        let level = match setting {
            MachineEnable::Inhibit => Level::Low,
            MachineEnable::Enable => Level::High,
        };
        output.set_level(level);
    }
}

pub(crate) fn panic(r: MachineEnableResources) {
    // Ensure the machine enable relay is turned off, disabling the Ruida controller from
    // attempting to operate the machine.
    Output::new(r.relay, Level::Low);
}
