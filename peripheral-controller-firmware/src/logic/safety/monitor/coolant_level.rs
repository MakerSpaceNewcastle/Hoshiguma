use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::cooler::{HEADER_TANK_COOLANT_LEVEL_CHANGED, HEAT_EXCHANGER_FLUID_LEVEL_CHANGED},
};
use defmt::unwrap;
use embassy_futures::select::{select, Either};
use hoshiguma_protocol::{
    cooler::types::HeatExchangeFluidLevel, peripheral_controller::types::MonitorKind,
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut header_tank_level_rx = HEADER_TANK_COOLANT_LEVEL_CHANGED.receiver().unwrap();
    let mut heat_exchanger_level_rx = HEAT_EXCHANGER_FLUID_LEVEL_CHANGED.receiver().unwrap();

    let mut header_tank_sensor_severity = Severity::Critical;
    let mut header_tank_level_severity = Severity::Critical;
    let mut heat_exchanger_level_severity = Severity::Critical;

    loop {
        match select(
            header_tank_level_rx.changed(),
            heat_exchanger_level_rx.changed(),
        )
        .await
        {
            Either::First(header_tank_reading) => {
                // TODO
            }
            Either::Second(heat_exchanger_reading) => {
                let severity = match heat_exchanger_reading {
                    HeatExchangeFluidLevel::Normal => Severity::Normal,
                    HeatExchangeFluidLevel::Low => Severity::Critical,
                };

                if checked_set(&mut heat_exchanger_level_severity, severity) == Changed::Yes {
                    status_tx
                        .publish((
                            MonitorKind::HeatExchangerFluidLow,
                            heat_exchanger_level_severity.clone(),
                        ))
                        .await;
                }
            }
        }

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
