use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::cooler::{
    HEADER_TANK_COOLANT_LEVEL_CHANGED, HEAT_EXCHANGER_FLUID_LEVEL_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::{
    cooler::types::{HeaderTankCoolantLevel, HeatExchangeFluidLevel},
    peripheral_controller::types::MonitorKind,
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn heat_exchanger_task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("hxf lvl mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut level_rx = HEAT_EXCHANGER_FLUID_LEVEL_CHANGED.receiver().unwrap();

    let mut level_low = ObservedSeverity::default();

    loop {
        let reading = level_rx.changed().await;

        level_low
            .update_and_async(
                match reading {
                    HeatExchangeFluidLevel::Normal => Severity::Normal,
                    HeatExchangeFluidLevel::Low => Severity::Critical,
                },
                |severity| async {
                    status_tx
                        .publish((MonitorKind::HeatExchangerFluidLow, severity))
                        .await;
                },
            )
            .await;
    }
}

#[embassy_executor::task]
pub(crate) async fn coolant_header_tank_task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cht lvl mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut level_rx = HEADER_TANK_COOLANT_LEVEL_CHANGED.receiver().unwrap();

    let mut sensor = ObservedSeverity::default();
    let mut level_low = ObservedSeverity::default();
    let mut level_high = ObservedSeverity::default();

    loop {
        let reading = level_rx.changed().await;

        sensor
            .update_and_async(
                match reading {
                    Ok(_) => Severity::Normal,
                    Err(_) => Severity::Warn,
                },
                |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolantHeaderTankLevelSensorFault, severity))
                        .await;
                },
            )
            .await;

        if let Ok(reading) = reading {
            level_low
                .update_and_async(
                    match reading {
                        HeaderTankCoolantLevel::Empty => Severity::Warn,
                        HeaderTankCoolantLevel::Normal => Severity::Normal,
                        HeaderTankCoolantLevel::Full => Severity::Normal,
                    },
                    |severity| async {
                        status_tx
                            .publish((MonitorKind::CoolantHeaderTankEmpty, severity))
                            .await;
                    },
                )
                .await;

            level_high
                .update_and_async(
                    match reading {
                        HeaderTankCoolantLevel::Empty => Severity::Normal,
                        HeaderTankCoolantLevel::Normal => Severity::Normal,
                        HeaderTankCoolantLevel::Full => Severity::Warn,
                    },
                    |severity| async {
                        status_tx
                            .publish((MonitorKind::CoolantHeaderTankOverfilled, severity))
                            .await;
                    },
                )
                .await;
        }
    }
}
