use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::machine_power_detector::MACHINE_POWER_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::{
    peripheral_controller::types::{MachinePower, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut power_changed_rx = unwrap!(MACHINE_POWER_CHANGED.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut severity = Severity::Critical;

    loop {
        let state = power_changed_rx.changed().await;

        let new_severity = match state {
            MachinePower::Off => Severity::Critical,
            MachinePower::On => Severity::Normal,
        };

        if checked_set(&mut severity, new_severity) == Changed::Yes {
            status_tx
                .publish((MonitorKind::LogicPowerSupplyNotPresent, severity.clone()))
                .await;
        }
    }
}
