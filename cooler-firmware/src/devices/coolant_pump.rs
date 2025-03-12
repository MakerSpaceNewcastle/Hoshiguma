use crate::{rpc::report_event, CoolantPumpResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::cooler::{
    event::{ControlEvent, EventKind},
    types::CoolantPump,
};

pub(crate) static COOLANT_PUMP: Watch<CriticalSectionRawMutex, CoolantPump, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: CoolantPumpResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(COOLANT_PUMP.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        report_event(EventKind::Control(ControlEvent::CoolantPump(
            setting.clone(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            CoolantPump::Idle => Level::Low,
            CoolantPump::Run => Level::High,
        };
        output.set_level(level);
    }
}
