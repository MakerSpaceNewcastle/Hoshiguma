use crate::io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use defmt::{unwrap, Format};
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[macro_export]
macro_rules! init_status_lamp {
    ($p:expr) => {{
        // Relay 0
        let red = Output::new($p.PIN_7, Level::Low);

        // Relay 1
        let amber = Output::new($p.PIN_6, Level::Low);

        // Relay 2
        let green = Output::new($p.PIN_16, Level::Low);

        $crate::devices::status_lamp::StatusLamp::new([red, amber, green])
    }};
}

pub(crate) type StatusLamp = DigitalOutputController<3, StatusLampSetting>;

#[derive(Default, Clone, Format)]
pub(crate) struct StatusLampSetting {
    pub(crate) red: bool,
    pub(crate) amber: bool,
    pub(crate) green: bool,
}

#[cfg(feature = "telemetry")]
impl From<&StatusLampSetting> for hoshiguma_telemetry_protocol::payload::control::StatusLamp {
    fn from(value: &StatusLampSetting) -> Self {
        Self {
            red: value.red,
            amber: value.amber,
            green: value.green,
        }
    }
}

impl StateToDigitalOutputs<3> for StatusLampSetting {
    fn to_outputs(self) -> [Level; 3] {
        [self.red.into(), self.amber.into(), self.green.into()]
    }
}

impl StatusLampSetting {
    pub(crate) fn set_red(&mut self, on: bool) {
        self.red = on;
    }

    pub(crate) fn set_amber(&mut self, on: bool) {
        self.amber = on;
    }

    pub(crate) fn set_green(&mut self, on: bool) {
        self.green = on;
    }
}

pub(crate) static STATUS_LAMP: Watch<CriticalSectionRawMutex, StatusLampSetting, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(mut status_lamp: StatusLamp) {
    let mut rx = unwrap!(STATUS_LAMP.receiver());

    loop {
        let setting = rx.changed().await;

        #[cfg(feature = "telemetry")]
        queue_telemetry_message(Payload::Control(ControlPayload::StatusLamp(
            (&setting).into(),
        )))
        .await;

        status_lamp.set(setting);
    }
}
