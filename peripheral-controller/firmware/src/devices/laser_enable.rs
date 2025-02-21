#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use crate::{
    io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs},
    LaserEnableResources,
};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

pub(crate) type LaserEnable = DigitalOutputController<1, LaserEnableState>;

impl From<LaserEnableResources> for LaserEnable {
    fn from(r: LaserEnableResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self::new([output])
    }
}

#[derive(Clone, Format)]
pub(crate) enum LaserEnableState {
    Inhibited,
    Enabled,
}

#[cfg(feature = "telemetry")]
impl From<&LaserEnableState> for hoshiguma_telemetry_protocol::payload::control::LaserEnable {
    fn from(value: &LaserEnableState) -> Self {
        match value {
            LaserEnableState::Inhibited => Self::Inhibited,
            LaserEnableState::Enabled => Self::Enabled,
        }
    }
}

impl StateToDigitalOutputs<1> for LaserEnableState {
    fn to_outputs(self) -> [Level; 1] {
        match self {
            Self::Inhibited => [Level::Low],
            Self::Enabled => [Level::High],
        }
    }
}

pub(crate) static LASER_ENABLE: Watch<CriticalSectionRawMutex, LaserEnableState, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: LaserEnableResources) {
    let mut laser_enable: LaserEnable = r.into();

    let mut rx = LASER_ENABLE.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        #[cfg(feature = "telemetry")]
        queue_telemetry_message(Payload::Control(ControlPayload::LaserEnable(
            (&setting).into(),
        )))
        .await;

        laser_enable.set(setting);
    }
}
