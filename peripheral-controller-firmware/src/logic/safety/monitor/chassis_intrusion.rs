use super::{MonitorStatusExt, NEW_MONITOR_STATUS};
use crate::{changed::Changed, devices::chassis_intrusion_detector::CHASSIS_INTRUSION_CHANGED};
use defmt::unwrap;
use hoshiguma_protocol::peripheral_controller::types::{
    ChassisIntrusion, Monitor, MonitorState, MonitorStatus,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut rx = unwrap!(CHASSIS_INTRUSION_CHANGED.receiver());

    let mut status = MonitorStatus::new(Monitor::ChassisIntrusion);

    loop {
        let state = rx.changed().await;

        let state = match state {
            ChassisIntrusion::Normal => MonitorState::Normal,
            ChassisIntrusion::Intruded => MonitorState::Critical,
        };

        if status.refresh(state) == Changed::Yes {
            NEW_MONITOR_STATUS.send(status.clone()).await;
        }
    }
}
