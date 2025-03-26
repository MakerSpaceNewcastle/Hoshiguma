pub(crate) mod chassis_intrusion;
pub(crate) mod coolant_level;
pub(crate) mod power;
pub(crate) mod temperatures;

use crate::{
    changed::{checked_set, Changed},
    telemetry::queue_telemetry_event,
};
use defmt::{debug, info, unwrap};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
    watch::Watch,
};
use hoshiguma_protocol::peripheral_controller::{event::EventKind, types::Monitors};
use hoshiguma_protocol::{peripheral_controller::types::MonitorKind, types::Severity};

static NEW_MONITOR_STATUS: PubSubChannel<
    CriticalSectionRawMutex,
    (MonitorKind, Severity),
    8,
    1,
    4,
> = PubSubChannel::new();

pub(crate) static MONITORS_CHANGED: Watch<CriticalSectionRawMutex, Monitors, 2> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn observation_task() {
    let mut rx = unwrap!(NEW_MONITOR_STATUS.subscriber());
    let tx = MONITORS_CHANGED.sender();

    let mut monitors = Monitors::default();

    loop {
        match rx.next_message().await {
            WaitResult::Lagged(n) => {
                panic!("Monitor observer channel lagged, losing {} messages", n)
            }
            WaitResult::Message(new_status) => {
                debug!("Monitor changed: {} -> {}", new_status.0, new_status.1);

                let severity = monitors.get_mut(new_status.0);
                if checked_set(severity, new_status.1) == Changed::Yes {
                    info!("Monitors changed: {}", monitors);
                    tx.send(monitors.clone());
                    queue_telemetry_event(EventKind::MonitorsChanged(monitors.clone())).await;
                }
            }
        }
    }
}
