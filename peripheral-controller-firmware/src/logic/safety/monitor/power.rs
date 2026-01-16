use super::{NEW_MONITOR_STATUS, ObservedSeverity};
use crate::devices::machine_power_detector::MACHINE_POWER_CHANGED;
use defmt::unwrap;
use hoshiguma_protocol::{
    peripheral_controller::types::{MachinePower, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("mach pwr mon").await;

    let mut power_changed_rx = unwrap!(MACHINE_POWER_CHANGED.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut severity = ObservedSeverity::default();

    loop {
        let state = power_changed_rx.changed().await;

        let new_severity = match state {
            MachinePower::Off => Severity::Critical,
            MachinePower::On => Severity::Normal,
        };

        severity
            .update_and_async(new_severity, |severity| async {
                status_tx
                    .publish((MonitorKind::MachinePowerOff, severity))
                    .await;
            })
            .await;
    }
}
