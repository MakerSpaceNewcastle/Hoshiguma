use crate::devices::{
    cooler::{CoolerControlCommand, COOLER_CONTROL},
    machine_power_detector::MACHINE_POWER_CHANGED,
};
use defmt::unwrap;
use hoshiguma_protocol::{
    cooler::types::{CoolantPump, Stirrer},
    peripheral_controller::types::MachinePower,
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
