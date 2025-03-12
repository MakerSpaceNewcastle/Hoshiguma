use crate::{rpc::report_event, StirrerResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::cooler::{
    event::{ControlEvent, EventKind},
    types::Stirrer,
};

pub(crate) static STIRRER: Watch<CriticalSectionRawMutex, Stirrer, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: StirrerResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(STIRRER.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        report_event(EventKind::Control(ControlEvent::Stirrer(setting.clone()))).await;

        // Set relay output
        let level = match setting {
            Stirrer::Idle => Level::Low,
            Stirrer::Run => Level::High,
        };
        output.set_level(level);
    }
}
