pub(crate) mod chassis_intrusion;
pub(crate) mod coolant_flow;
pub(crate) mod coolant_level;
pub(crate) mod extraction_airflow;
pub(crate) mod power;
pub(crate) mod temperatures_a;
pub(crate) mod temperatures_b;

use crate::{
    changed::{checked_set, Changed, ObservedValue},
    telemetry::queue_telemetry_event,
};
use defmt::{debug, info, unwrap, warn};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
    watch::Watch,
};
use hoshiguma_protocol::{
    peripheral_controller::{
        event::EventKind,
        types::{MonitorKind, Monitors},
    },
    types::{Severity, TemperatureReading},
};

pub(crate) static NEW_MONITOR_STATUS: PubSubChannel<
    CriticalSectionRawMutex,
    (MonitorKind, Severity),
    8,
    1,
    9,
> = PubSubChannel::new();

pub(crate) static MONITORS_CHANGED: Watch<CriticalSectionRawMutex, Monitors, 3> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn observation_task() {
    let mut rx = unwrap!(NEW_MONITOR_STATUS.subscriber());
    let tx = MONITORS_CHANGED.sender();

    let mut monitors = Monitors::default();

    loop {
        match rx.next_message().await {
            WaitResult::Message(new_status) => {
                debug!("Monitor changed: {} -> {}", new_status.0, new_status.1);

                let severity = monitors.get_mut(new_status.0);
                if checked_set(severity, new_status.1) == Changed::Yes {
                    info!("Monitors changed: {}", monitors);
                    tx.send(monitors.clone());
                    queue_telemetry_event(EventKind::MonitorsChanged(monitors.clone())).await;
                }
            }
            WaitResult::Lagged(msg_count) => {
                panic!("Subscriber lagged, losing {} messages", msg_count);
            }
        }
    }
}

pub(crate) type ObservedSeverity = ObservedValue<Severity>;

fn temperature_to_state(
    warn: f32,
    critical: f32,
    temperature: TemperatureReading,
) -> Result<Severity, ()> {
    if let Ok(temperature) = temperature {
        Ok(if temperature >= critical {
            warn!(
                "Temperature {} is above critical threshold of {}",
                temperature, critical
            );
            Severity::Critical
        } else if temperature >= warn {
            warn!(
                "Temperature {} is above warning threshold of {}",
                temperature, warn
            );
            Severity::Warn
        } else {
            debug!("Temperature {} is normal", temperature);
            Severity::Normal
        })
    } else {
        warn!("Asked to check temperature of a sensor that failed to be read");
        Err(())
    }
}
