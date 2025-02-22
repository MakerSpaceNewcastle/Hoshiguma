use crate::{
    io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs},
    telemetry::queue_telemetry_message,
    MachineEnableResources,
};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

pub(crate) type MachineEnable = DigitalOutputController<1, MachineEnableState>;

impl From<MachineEnableResources> for MachineEnable {
    fn from(r: MachineEnableResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self::new([output])
    }
}

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

impl StateToDigitalOutputs<1> for MachineEnableState {
    fn to_outputs(self) -> [Level; 1] {
        match self {
            Self::Inhibited => [Level::Low],
            Self::Enabled => [Level::High],
        }
    }
}

pub(crate) static MACHINE_ENABLE: Watch<CriticalSectionRawMutex, MachineEnableState, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineEnableResources) {
    let mut machine_enable: MachineEnable = r.into();

    let mut rx = MACHINE_ENABLE.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        queue_telemetry_message(Payload::Control(ControlPayload::MachineEnable(
            (&setting).into(),
        )))
        .await;

        machine_enable.set(setting);
    }
}
