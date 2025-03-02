use crate::{telemetry::queue_telemetry_message, AirAssistPumpResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{
    control::{AirAssistPump, ControlPayload},
    Payload,
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
        queue_telemetry_message(Payload::Control(ControlPayload::AirAssistPump(
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
