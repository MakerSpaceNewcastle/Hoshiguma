use crate::{telemetry::queue_telemetry_message, MachineEnableResources};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum MachineEnableState {
    Inhibited,
    Enabled,
}

impl From<&MachineEnableState> for hoshiguma_telemetry_protocol::payload::control::MachineEnable {
    fn from(value: &MachineEnableState) -> Self {
        match value {
            MachineEnableState::Inhibited => Self::Inhibited,
            MachineEnableState::Enabled => Self::Enabled,
        }
    }
}

pub(crate) static MACHINE_ENABLE: Watch<CriticalSectionRawMutex, MachineEnableState, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = MACHINE_ENABLE.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::MachineEnable(
            (&setting).into(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            MachineEnableState::Inhibited => Level::Low,
            MachineEnableState::Enabled => Level::High,
        };
        output.set_level(level);
    }
}

pub(crate) fn panic(r: MachineEnableResources) {
    // Ensure the machine enable relay is turned off, disabling the Ruida controller from
    // attempting to operate the machine.
    Output::new(r.relay, Level::Low);
}
