use crate::{telemetry::queue_telemetry_event, StatusLampResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind},
    types::StatusLamp,
};

pub(crate) struct StatusLampOutput {
    red: Output<'static>,
    amber: Output<'static>,
    green: Output<'static>,
}

impl StatusLampOutput {
    pub(crate) fn new(r: StatusLampResources) -> Self {
        let red = Output::new(r.red, Level::Low);
        let amber = Output::new(r.amber, Level::Low);
        let green = Output::new(r.green, Level::Low);
        Self { red, amber, green }
    }

    pub(crate) fn set(&mut self, setting: StatusLamp) {
        self.red.set_level(lamp_on_to_level(setting.red));
        self.amber.set_level(lamp_on_to_level(setting.amber));
        self.green.set_level(lamp_on_to_level(setting.green));
    }

    /// Set all lights to on, should never happen under normal circumstances (and is labelled on the
    /// light pillar as a "something is very wrong" indication).
    #[cfg(not(feature = "panic-probe"))]
    pub(crate) fn set_panic(&mut self) {
        self.red.set_high();
        self.amber.set_high();
        self.green.set_high();
    }
}

fn lamp_on_to_level(on: bool) -> Level {
    match on {
        true => Level::High,
        false => Level::Low,
    }
}

pub(crate) static STATUS_LAMP: Watch<CriticalSectionRawMutex, StatusLamp, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: StatusLampResources) {
    let mut output = StatusLampOutput::new(r);
    let mut rx = unwrap!(STATUS_LAMP.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_event(EventKind::Control(ControlEvent::StatusLamp(
            setting.clone(),
        )))
        .await;

        // Set relay output
        output.set(setting);
    }
}
