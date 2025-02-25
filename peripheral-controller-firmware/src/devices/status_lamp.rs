use crate::{telemetry::queue_telemetry_message, StatusLampResources};
use defmt::{unwrap, Format};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[derive(Default, Clone, Format)]
pub(crate) struct StatusLampSetting {
    red: bool,
    amber: bool,
    green: bool,
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
    let mut red = Output::new(r.red, Level::Low);
    let mut amber = Output::new(r.amber, Level::Low);
    let mut green = Output::new(r.green, Level::Low);

    let mut rx = unwrap!(STATUS_LAMP.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::StatusLamp(
            (&setting).into(),
        )))
        .await;

        // Set relay output
        red.set_level(lamp_on_to_level(setting.red));
        amber.set_level(lamp_on_to_level(setting.amber));
        green.set_level(lamp_on_to_level(setting.green));
    }
}

fn lamp_on_to_level(on: bool) -> Level {
    match on {
        true => Level::High,
        false => Level::Low,
    }
}

pub(crate) fn panic(r: StatusLampResources) {
    // Set all lights to on, should never happen under normal circumstances (and is labelled on the
    // light pillar as a "something is very wrong" indication).
    Output::new(r.red, Level::High);
    Output::new(r.amber, Level::High);
    Output::new(r.green, Level::High);
}
