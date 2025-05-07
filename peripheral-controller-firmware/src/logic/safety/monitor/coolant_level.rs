use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::cooler::COOLANT_RESEVOIR_LEVEL_CHANGED;
use defmt::unwrap;
use hoshiguma_protocol::{
    cooler::types::CoolantReservoirLevel, peripheral_controller::types::MonitorKind,
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("coolant res level mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut level_rx = COOLANT_RESEVOIR_LEVEL_CHANGED.receiver().unwrap();

    let mut level_low = ObservedSeverity::default();

    loop {
        let reading = level_rx.changed().await;

        level_low
            .update_and_async(
                match reading {
                    CoolantReservoirLevel::Normal => Severity::Normal,
                    CoolantReservoirLevel::Low => Severity::Critical,
                },
                |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolantReservoirLevelLow, severity))
                        .await;
                },
            )
            .await;
    }
}
