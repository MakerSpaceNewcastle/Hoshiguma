use crate::io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[macro_export]
macro_rules! init_machine_enable {
    ($p:expr) => {{
        // Relay output 3
        let output = embassy_rp::gpio::Output::new($p.PIN_17, embassy_rp::gpio::Level::Low);

        $crate::devices::machine_enable::MachineEnable::new([output])
    }};
}

pub(crate) type MachineEnable = DigitalOutputController<1, MachineEnableState>;

#[derive(Clone, Format)]
pub(crate) enum MachineEnableState {
    Inhibited,
    Enabled,
}

#[cfg(feature = "telemetry")]
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
pub(crate) async fn task(mut machine_enable: MachineEnable) {
    let mut rx = MACHINE_ENABLE.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        #[cfg(feature = "telemetry")]
        queue_telemetry_message(Payload::Control(ControlPayload::MachineEnable(
            (&setting).into(),
        )))
        .await;

        machine_enable.set(setting);
    }
}
