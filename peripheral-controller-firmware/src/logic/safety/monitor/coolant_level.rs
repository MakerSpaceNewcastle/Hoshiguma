use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::cooler::HEADER_TANK_COOLANT_LEVEL_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::{
    peripheral_controller::types::{CoolantResevoirLevel, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let header_tank_level_rx = HEADER_TANK_COOLANT_LEVEL_CHANGED.receiver().unwrap();

    let mut sensor_severity = Severity::Critical;
    let mut level_severity = Severity::Critical;

    loop {
        todo!();

        // let new_sensor_severity = if state.is_ok() {
        //     Severity::Normal
        // } else {
        //     Severity::Warn
        // };

        // if checked_set(&mut sensor_severity, new_sensor_severity) == Changed::Yes {
        //     status_tx
        //         .publish((
        //             MonitorKind::CoolantResevoirLevelSensorFault,
        //             sensor_severity.clone(),
        //         ))
        //         .await;
        // }

        // if let Ok(level_state) = state {
        //     let new_level_severity = match level_state {
        //         CoolantResevoirLevel::Full => Severity::Normal,
        //         CoolantResevoirLevel::Low => Severity::Warn,
        //         CoolantResevoirLevel::Empty => Severity::Critical,
        //     };

        //     if checked_set(&mut level_severity, new_level_severity) == Changed::Yes {
        //         status_tx
        //             .publish((MonitorKind::CoolantResevoirLevel, level_severity.clone()))
        //             .await;
        //     }
        // }
    }
}
