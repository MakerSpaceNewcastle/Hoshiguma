use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::chassis_intrusion_detector::CHASSIS_INTRUSION_CHANGED,
};
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

    let mut severity = Severity::Critical;

    loop {
        let state = intrusion_rx.changed().await;

        let new_severity = match state {
            ChassisIntrusion::Normal => Severity::Normal,
            ChassisIntrusion::Intruded => Severity::Critical,
        };

        if checked_set(&mut severity, new_severity) == Changed::Yes {
            status_tx
                .publish((MonitorKind::ChassisIntrusion, severity.clone()))
                .await;
        }
    }
}
