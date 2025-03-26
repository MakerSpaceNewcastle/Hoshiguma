use super::monitor::MONITORS_CHANGED;
use crate::{
    devices::{
        laser_enable::LASER_ENABLE, machine_enable::MACHINE_ENABLE,
        machine_run_detector::MACHINE_RUNNING_CHANGED,
    },
    telemetry::queue_telemetry_event,
};
use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::{
    peripheral_controller::{
        event::EventKind,
        types::{LaserEnable, MachineEnable, MachineOperationLockout, MachineRun},
    },
    types::Severity,
};

pub(crate) static MACHINE_LOCKOUT_CHANGED: Watch<
    CriticalSectionRawMutex,
    MachineOperationLockout,
    3,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn alarm_evaluation_task() {
    let mut running_rx = MACHINE_RUNNING_CHANGED.receiver().unwrap();
    let mut monitors_rx = MONITORS_CHANGED.receiver().unwrap();

    let machine_lockout_tx = MACHINE_LOCKOUT_CHANGED.sender();

    let mut is_running = MachineRun::Idle;
    let mut severity = Severity::Critical;

    loop {
        use embassy_futures::select::{select, Either};

        match select(running_rx.changed(), monitors_rx.changed()).await {
            Either::First(running) => {
                is_running = running;
            }
            Either::Second(monitors) => {
                severity = monitors.severity();
            }
        }

        let lockout = match severity {
            Severity::Normal => MachineOperationLockout::Permitted,
            Severity::Warn => match is_running {
                MachineRun::Idle => MachineOperationLockout::Denied,
                MachineRun::Running => MachineOperationLockout::PermittedUntilIdle,
            },
            Severity::Critical => MachineOperationLockout::Denied,
        };
        info!("Machine operation lockout: {}", lockout);

        machine_lockout_tx.send(lockout.clone());
        queue_telemetry_event(EventKind::LockoutChanged(lockout)).await;
    }
}

#[embassy_executor::task]
pub(crate) async fn machine_lockout_task() {
    let mut machine_locout_rx = MACHINE_LOCKOUT_CHANGED.receiver().unwrap();

    let machine_enable_tx = MACHINE_ENABLE.sender();
    let laser_enable_tx = LASER_ENABLE.sender();

    loop {
        let lockout = machine_locout_rx.changed().await;

        machine_enable_tx.send(match lockout {
            MachineOperationLockout::Permitted => MachineEnable::Enable,
            MachineOperationLockout::PermittedUntilIdle => MachineEnable::Enable,
            MachineOperationLockout::Denied => MachineEnable::Inhibit,
        });

        laser_enable_tx.send(match lockout {
            MachineOperationLockout::Permitted => LaserEnable::Enable,
            MachineOperationLockout::PermittedUntilIdle => LaserEnable::Enable,
            MachineOperationLockout::Denied => LaserEnable::Inhibit,
        });
    }
}
