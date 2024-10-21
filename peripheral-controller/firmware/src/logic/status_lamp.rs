use crate::{
    devices::{
        machine_power_detector::{MachinePower, MACHINE_POWER_CHANGED},
        machine_run_detector::{MachineRunStatus, MACHINE_RUNNING_CHANGED},
        status_lamp::{StatusLampSetting, STATUS_LAMP},
    },
    logic::safety::lockout::{MachineOperationLockout, MACHINE_LOCKOUT_CHANGED},
};
use defmt::unwrap;

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut machine_power_rx = unwrap!(MACHINE_POWER_CHANGED.receiver());
    let mut running_rx = unwrap!(MACHINE_RUNNING_CHANGED.receiver());
    let mut operation_lockout_rx = unwrap!(MACHINE_LOCKOUT_CHANGED.receiver());

    let status_lamp_tx = STATUS_LAMP.sender();

    let mut lamp = StatusLampSetting::default();
    let mut machine_power = MachinePower::Off;

    loop {
        use embassy_futures::select::{select3, Either3};

        match select3(
            machine_power_rx.changed(),
            running_rx.changed(),
            operation_lockout_rx.changed(),
        )
        .await
        {
            Either3::First(power) => {
                machine_power = power;
            }
            Either3::Second(running) => {
                // The amber light is lit when the machine is running
                lamp.set_amber(match running {
                    MachineRunStatus::Idle => false,
                    MachineRunStatus::Running => true,
                });
            }
            Either3::Third(lockout) => {
                // The red lamp is lit when operation of the machine is denied
                lamp.set_red(match lockout {
                    MachineOperationLockout::Permitted => false,
                    MachineOperationLockout::PermittedUntilIdle => false,
                    MachineOperationLockout::Denied => true,
                });

                // The green lamp is lit when operation of the machine is permitted and will
                // continue to be permitted
                lamp.set_green(match lockout {
                    MachineOperationLockout::Permitted => true,
                    MachineOperationLockout::PermittedUntilIdle => false,
                    MachineOperationLockout::Denied => false,
                });
            }
        }

        // Turn off all lamps if the 24V bus is not powered
        // (the lamps would not be lit anyway, this is to save realys being powered constantly for
        // no reason)
        status_lamp_tx.send(match machine_power {
            MachinePower::Off => StatusLampSetting::default(),
            MachinePower::On => lamp.clone(),
        });
    }
}
