use crate::CompressorResources;
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::cooler::types::Compressor;

pub(crate) static COMPRESSOR: Watch<CriticalSectionRawMutex, Compressor, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: CompressorResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(COMPRESSOR.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        // TODO
        // queue_telemetry_event(EventKind::Control(ControlEvent::AirAssistPump(
        //     setting.clone(),
        // )))
        // .await;

        // Set relay output
        let level = match setting {
            Compressor::Idle => Level::Low,
            Compressor::Run => Level::High,
        };
        output.set_level(level);
    }
}
