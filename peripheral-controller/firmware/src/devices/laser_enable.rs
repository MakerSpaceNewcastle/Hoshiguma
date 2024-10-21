use crate::io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[macro_export]
macro_rules! init_laser_enable {
    ($p:expr) => {{
        // Relay output 4
        let output = embassy_rp::gpio::Output::new($p.PIN_18, embassy_rp::gpio::Level::Low);

        $crate::devices::laser_enable::LaserEnable::new([output])
    }};
}

pub(crate) type LaserEnable = DigitalOutputController<1, LaserEnableState>;

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
pub(crate) async fn task(mut laser_enable: LaserEnable) {
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
