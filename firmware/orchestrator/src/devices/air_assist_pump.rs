use crate::{AirAssistPumpResources, telemetry::queue_telemetry_data_point};
use defmt::unwrap;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_core::{telemetry::AsTelemetry, types::AirAssistPump};
use pico_plc_bsp::embassy_rp::gpio::{Level, Output};

pub(crate) static AIR_ASSIST_PUMP: Watch<CriticalSectionRawMutex, AirAssistPump, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: AirAssistPumpResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("aa pump o/p").await;

    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(AIR_ASSIST_PUMP.receiver());

    loop {
        let setting = rx.changed().await;

        for dp in setting.telemetry() {
            queue_telemetry_data_point(dp);
        }

        let level = match setting {
            AirAssistPump::Idle => Level::Low,
            AirAssistPump::Run => Level::High,
        };
        output.set_level(level);
    }
}
