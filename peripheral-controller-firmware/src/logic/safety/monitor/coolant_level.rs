use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::coolant_resevoir_level_sensor::COOLANT_RESEVOIR_LEVEL_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::{
    peripheral_controller::types::{CoolantResevoirLevel, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut rx = unwrap!(COOLANT_RESEVOIR_LEVEL_CHANGED.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut sensor_severity = Severity::Critical;
    let mut level_severity = Severity::Critical;

    loop {
        let state = rx.changed().await;

        let new_sensor_severity = if state.is_ok() {
            Severity::Normal
        } else {
            Severity::Warn
        };

        if checked_set(&mut sensor_severity, new_sensor_severity) == Changed::Yes {
            status_tx
                .publish((
                    MonitorKind::CoolantResevoirLevelSensorFault,
                    sensor_severity.clone(),
                ))
                .await;
        }

        if let Ok(level_state) = state {
            let new_level_severity = match level_state {
                CoolantResevoirLevel::Full => Severity::Normal,
                CoolantResevoirLevel::Low => Severity::Warn,
                CoolantResevoirLevel::Empty => Severity::Critical,
            };

            if checked_set(&mut level_severity, new_level_severity) == Changed::Yes {
                status_tx
                    .publish((MonitorKind::CoolantResevoirLevel, level_severity.clone()))
                    .await;
            }
        }
    }
}
