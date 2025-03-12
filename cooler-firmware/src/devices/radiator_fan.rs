use crate::{rpc::report_event, RadiatorFanResources};
use defmt::unwrap;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::cooler::{
    event::{ControlEvent, EventKind},
    types::RadiatorFan,
};

pub(crate) static RADIATOR_FAN: Watch<CriticalSectionRawMutex, RadiatorFan, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: RadiatorFanResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(RADIATOR_FAN.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        report_event(EventKind::Control(ControlEvent::RadiatorFan(
            setting.clone(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            RadiatorFan::Idle => Level::Low,
            RadiatorFan::Run => Level::High,
        };
        output.set_level(level);
    }
}
