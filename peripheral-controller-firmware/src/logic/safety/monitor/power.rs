use super::{MonitorStatusExt, NEW_MONITOR_STATUS};
use crate::{changed::Changed, devices::machine_power_detector::MACHINE_POWER_CHANGED};
use defmt::unwrap;
use hoshiguma_protocol::payload::{
    observation::MachinePower,
    process::{Monitor, MonitorState, MonitorStatus},
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut rx = unwrap!(MACHINE_POWER_CHANGED.receiver());

    let mut status = MonitorStatus::new(Monitor::LogicPowerSupplyNotPresent);

    loop {
        let state = rx.changed().await;

        let state = match state {
            MachinePower::Off => MonitorState::Critical,
            MachinePower::On => MonitorState::Normal,
        };

        if status.refresh(state) == Changed::Yes {
            NEW_MONITOR_STATUS.send(status.clone()).await;
        }
    }
}
