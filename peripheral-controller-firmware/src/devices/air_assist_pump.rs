use crate::{telemetry::queue_telemetry_event, AirAssistPumpResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind},
    types::AirAssistPump,
};

pub(crate) static AIR_ASSIST_PUMP: Watch<CriticalSectionRawMutex, AirAssistPump, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: AirAssistPumpResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(AIR_ASSIST_PUMP.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_event(EventKind::Control(ControlEvent::AirAssistPump(
            setting.clone(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            AirAssistPump::Idle => Level::Low,
            AirAssistPump::Run => Level::High,
        };
        output.set_level(level);
    }
}
