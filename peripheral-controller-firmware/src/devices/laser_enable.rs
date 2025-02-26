use crate::{telemetry::queue_telemetry_message, LaserEnableResources};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum LaserEnableState {
    Inhibited,
    Enabled,
}

impl From<&LaserEnableState> for hoshiguma_telemetry_protocol::payload::control::LaserEnable {
    fn from(value: &LaserEnableState) -> Self {
        match value {
            LaserEnableState::Inhibited => Self::Inhibited,
            LaserEnableState::Enabled => Self::Enabled,
        }
    }
}

pub(crate) static LASER_ENABLE: Watch<CriticalSectionRawMutex, LaserEnableState, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: LaserEnableResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = LASER_ENABLE.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::LaserEnable(
            (&setting).into(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            LaserEnableState::Inhibited => Level::Low,
            LaserEnableState::Enabled => Level::High,
        };
        output.set_level(level);
    }
}

pub(crate) fn panic(r: LaserEnableResources) {
    // Ensure the laser enable relay is turned off, disabling the laser power supply from turning
    // on.
    Output::new(r.relay, Level::Low);
}
