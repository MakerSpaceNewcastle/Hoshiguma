use crate::{
    io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs},
    telemetry::queue_telemetry_message,
    StatusLampResources,
};
use defmt::{unwrap, Format};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

pub(crate) type StatusLamp = DigitalOutputController<3, StatusLampSetting>;

impl From<StatusLampResources> for StatusLamp {
    fn from(r: StatusLampResources) -> Self {
        let red = Output::new(r.red, Level::Low);
        let amber = Output::new(r.amber, Level::Low);
        let green = Output::new(r.green, Level::Low);

        Self::new([red, amber, green])
    }
}

#[derive(Default, Clone, Format)]
pub(crate) struct StatusLampSetting {
    pub(crate) red: bool,
    pub(crate) amber: bool,
    pub(crate) green: bool,
}

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
pub(crate) async fn task(r: StatusLampResources) {
    let mut status_lamp: StatusLamp = r.into();
    let mut rx = unwrap!(STATUS_LAMP.receiver());

    loop {
        let setting = rx.changed().await;

        queue_telemetry_message(Payload::Control(ControlPayload::StatusLamp(
            (&setting).into(),
        )))
        .await;

        status_lamp.set(setting);
    }
}
