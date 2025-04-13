use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::cooler::COOLANT_FLOW_READ;
use defmt::unwrap;
use hoshiguma_protocol::{peripheral_controller::types::MonitorKind, types::Severity};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("clt flw mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut level_rx = COOLANT_FLOW_READ.receiver().unwrap();

    let mut severity = ObservedSeverity::default();

    loop {
        let reading = level_rx.changed().await;

        severity
            .update_and_async(
                if *reading < 0.05 {
                    Severity::Critical
                } else {
                    Severity::Normal
                },
                |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolantFlowInsufficient, severity))
                        .await;
                },
            )
            .await;
    }
}
