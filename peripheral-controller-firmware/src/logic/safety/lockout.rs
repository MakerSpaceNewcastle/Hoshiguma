use super::{alarms::ACTIVE_ALARMS_CHANGED, monitor::MonitorState};
use crate::{
    devices::{
        laser_enable::{LaserEnableState, LASER_ENABLE},
        machine_enable::{MachineEnableState, MACHINE_ENABLE},
        machine_run_detector::{MachineRunStatus, MACHINE_RUNNING_CHANGED},
    },
    telemetry::queue_telemetry_message,
};
use defmt::{info, Format};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{process::ProcessPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}

impl From<&MachineOperationLockout>
    for hoshiguma_telemetry_protocol::payload::process::MachineOperationLockout
{
    fn from(value: &MachineOperationLockout) -> Self {
        match value {
            MachineOperationLockout::Permitted => Self::Permitted,
            MachineOperationLockout::PermittedUntilIdle => Self::PermittedUntilIdle,
            MachineOperationLockout::Denied => Self::Denied,
        }
    }
}

pub(crate) static MACHINE_LOCKOUT_CHANGED: Watch<
    CriticalSectionRawMutex,
    MachineOperationLockout,
    3,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn alarm_evaluation_task() {
    let mut running_rx = MACHINE_RUNNING_CHANGED.receiver().unwrap();
    let mut active_alarms_rx = ACTIVE_ALARMS_CHANGED.receiver().unwrap();

    let machine_lockout_tx = MACHINE_LOCKOUT_CHANGED.sender();

    let mut is_running = MachineRunStatus::Idle;
    let mut alarm_state = MonitorState::Normal;

    loop {
        use embassy_futures::select::{select, Either};

        match select(running_rx.changed(), active_alarms_rx.changed()).await {
            Either::First(running) => {
                is_running = running;
            }
            Either::Second(alarms) => {
                alarm_state = alarms.overall_state();
            }
        }

        let lockout = match alarm_state {
            MonitorState::Normal => MachineOperationLockout::Permitted,
            MonitorState::Warn => match is_running {
                MachineRunStatus::Idle => MachineOperationLockout::Denied,
                MachineRunStatus::Running => MachineOperationLockout::PermittedUntilIdle,
            },
            MonitorState::Critical => MachineOperationLockout::Denied,
        };
        info!("Machine operation lockout: {}", lockout);

        queue_telemetry_message(Payload::Process(ProcessPayload::Lockout((&lockout).into()))).await;

        machine_lockout_tx.send(lockout);
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
            MachineOperationLockout::Permitted => MachineEnableState::Enabled,
            MachineOperationLockout::PermittedUntilIdle => MachineEnableState::Enabled,
            MachineOperationLockout::Denied => MachineEnableState::Inhibited,
        });

        laser_enable_tx.send(match lockout {
            MachineOperationLockout::Permitted => LaserEnableState::Enabled,
            MachineOperationLockout::PermittedUntilIdle => LaserEnableState::Enabled,
            MachineOperationLockout::Denied => LaserEnableState::Inhibited,
        });
    }
}
