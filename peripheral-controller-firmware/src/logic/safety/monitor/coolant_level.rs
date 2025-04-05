use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::cooler::{
    HEADER_TANK_COOLANT_LEVEL_CHANGED, HEAT_EXCHANGER_FLUID_LEVEL_CHANGED,
};
use defmt::unwrap;
use embassy_futures::select::{select, Either};
use hoshiguma_protocol::{
    cooler::types::{HeaderTankCoolantLevel, HeatExchangeFluidLevel},
    peripheral_controller::types::MonitorKind,
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut header_tank_level_rx = HEADER_TANK_COOLANT_LEVEL_CHANGED.receiver().unwrap();
    let mut heat_exchanger_level_rx = HEAT_EXCHANGER_FLUID_LEVEL_CHANGED.receiver().unwrap();

    let mut header_tank_sensor = ObservedSeverity::default();
    let mut header_tank_level_low = ObservedSeverity::default();
    let mut header_tank_level_high = ObservedSeverity::default();
    let mut heat_exchanger_level = ObservedSeverity::default();

    loop {
        match select(
            header_tank_level_rx.changed(),
            heat_exchanger_level_rx.changed(),
        )
        .await
        {
            Either::First(header_tank_reading) => {
                header_tank_sensor
                    .update_and_async(
                        match header_tank_reading {
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

                if let Ok(reading) = header_tank_reading {
                    header_tank_level_low
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

                    header_tank_level_high
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
            Either::Second(heat_exchanger_reading) => {
                heat_exchanger_level
                    .update_and_async(
                        match heat_exchanger_reading {
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
    }
}
