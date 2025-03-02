use super::{MonitorStatusExt, NEW_MONITOR_STATUS};
use crate::{
    changed::Changed, devices::coolant_resevoir_level_sensor::COOLANT_RESEVOIR_LEVEL_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::payload::{
    observation::CoolantResevoirLevel,
    process::{Monitor, MonitorState, MonitorStatus},
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut rx = unwrap!(COOLANT_RESEVOIR_LEVEL_CHANGED.receiver());

    let mut sensor_status = MonitorStatus::new(Monitor::CoolantResevoirLevelSensorFault);
    let mut level_status = MonitorStatus::new(Monitor::CoolantResevoirLevel);

    loop {
        let state = rx.changed().await;

        let sensor_state = if state.is_ok() {
            MonitorState::Normal
        } else {
            MonitorState::Warn
        };

        if sensor_status.refresh(sensor_state) == Changed::Yes {
            NEW_MONITOR_STATUS.send(sensor_status.clone()).await;
        }

        if let Ok(level_state) = state {
            let level_state = match level_state {
                CoolantResevoirLevel::Full => MonitorState::Normal,
                CoolantResevoirLevel::Low => MonitorState::Warn,
                CoolantResevoirLevel::Empty => MonitorState::Critical,
            };

            if level_status.refresh(level_state) == Changed::Yes {
                NEW_MONITOR_STATUS.send(level_status.clone()).await;
            }
        }
    }
}
