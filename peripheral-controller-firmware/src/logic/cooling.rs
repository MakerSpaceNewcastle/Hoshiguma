use crate::devices::{
    cooler::{CoolerControlCommand, COOLER_CONTROL},
    machine_power_detector::MACHINE_POWER_CHANGED,
};
use defmt::unwrap;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::{
    cooler::types::{Compressor, CoolantPump, RadiatorFan, Stirrer},
    peripheral_controller::types::{CoolingDemand, MachinePower},
};

#[embassy_executor::task]
pub(crate) async fn power_control() {
    let mut machine_power_rx = MACHINE_POWER_CHANGED.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL.publisher());

    loop {
        let machine_power = machine_power_rx.changed().await;

        match machine_power {
            MachinePower::On => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCoolantPump(CoolantPump::Run))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetStirrer(Stirrer::Run))
                    .await;
            }
            MachinePower::Off => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCoolantPump(CoolantPump::Idle))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetStirrer(Stirrer::Idle))
                    .await;
            }
        }
    }
}

pub(crate) static COOLING_DEMAND: Watch<CriticalSectionRawMutex, CoolingDemand, 1> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn cooling_control() {
    let mut cooling_demand_rx = COOLING_DEMAND.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL.publisher());

    loop {
        let cooling_demand = cooling_demand_rx.changed().await;

        match cooling_demand {
            CoolingDemand::Demand => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Run))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Run))
                    .await;
            }
            CoolingDemand::Idle => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Idle))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Idle))
                    .await;
            }
        }
    }
}
