use super::{NEW_MONITOR_STATUS, ObservedSeverity};
use crate::devices::accessories::cooler::COOLANT_RESERVOIR_LEVEL_CHANGED;
use defmt::unwrap;
use hoshiguma_core::{
    accessories::cooler::types::CoolantReservoirLevel,
    types::{MonitorKind, Severity},
};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("coolant res level mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut level_rx = COOLANT_RESERVOIR_LEVEL_CHANGED.receiver().unwrap();

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
