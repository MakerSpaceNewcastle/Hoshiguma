use crate::{
    devices::{
        cooler::{CoolerControlCommand, COOLER_CONTROL_ACK, COOLER_CONTROL_COMMAND},
        machine_power_detector::MACHINE_POWER_CHANGED,
    },
    telemetry::queue_telemetry_event,
};
use defmt::unwrap;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::{
    cooler::types::{Compressor, CoolantPump, RadiatorFan, Stirrer},
    peripheral_controller::{
        event::EventKind,
        types::{CoolingDemand, CoolingEnabled, MachinePower},
    },
};

#[embassy_executor::task]
pub(crate) async fn power_control() {
    let mut machine_power_rx = MACHINE_POWER_CHANGED.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL_COMMAND.publisher());

    // TODO: validation of cooler state

    loop {
        let power = machine_power_rx.changed().await;

        match power {
            MachinePower::On => {
                queue_telemetry_event(EventKind::CoolingEnableChanged(CoolingEnabled::Enable))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCoolantPump(CoolantPump::Run))
                    .await;
                // TODO: delay
                cooler_command_tx
                    .publish(CoolerControlCommand::SetStirrer(Stirrer::Run))
                    .await;
            }
            MachinePower::Off => {
                queue_telemetry_event(EventKind::CoolingEnableChanged(CoolingEnabled::Inhibit))
                    .await;
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCoolantPump(CoolantPump::Idle))
                    .await;
                // TODO: delay
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

    let cooler_command_tx = unwrap!(COOLER_CONTROL_COMMAND.publisher());
    let cooler_ack_rx = unwrap!(COOLER_CONTROL_ACK.subscriber());

    // TODO: validation of cooler state

    loop {
        let demand = cooling_demand_rx.changed().await;
        queue_telemetry_event(EventKind::CoolingDemandChanged(demand.clone())).await;

        match demand {
            CoolingDemand::Demand => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Run))
                    .await;
                // TODO: delay
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Run))
                    .await;
            }
            CoolingDemand::Idle => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Idle))
                    .await;
                // TODO: delay
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Idle))
                    .await;
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn thermal_monitor() {
    todo!();
}
