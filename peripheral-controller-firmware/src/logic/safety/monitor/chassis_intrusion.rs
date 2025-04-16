use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::chassis_intrusion_detector::CHASSIS_INTRUSION_CHANGED;
use defmt::unwrap;
use hoshiguma_protocol::{
    peripheral_controller::types::{ChassisIntrusion, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("chs int mon").await;

    let mut intrusion_rx = unwrap!(CHASSIS_INTRUSION_CHANGED.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut severity = ObservedSeverity::default();

    loop {
        let state = intrusion_rx.changed().await;

        let new_severity = match state {
            ChassisIntrusion::Normal => Severity::Normal,
            ChassisIntrusion::Intruded => Severity::Critical,
        };

        severity
            .update_and_async(new_severity, |severity| async {
                status_tx
                    .publish((MonitorKind::ChassisIntrusion, severity))
                    .await;
            })
            .await;
    }
}
