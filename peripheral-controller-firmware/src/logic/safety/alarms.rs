use super::monitor::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    telemetry::queue_telemetry_message,
};
use defmt::{debug, info, unwrap};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{
    process::{ActiveAlarms, MonitorState, MonitorStatus, ProcessPayload},
    Payload,
};

pub(super) trait ActiveAlarmsExt {
    fn update(&mut self, status: MonitorStatus) -> Changed;
    fn overall_state(&self) -> MonitorState;
}

impl ActiveAlarmsExt for ActiveAlarms {
    fn update(&mut self, status: MonitorStatus) -> Changed {
        match status.state {
            MonitorState::Normal => {
                let old_len = self.alarms.len();
                self.alarms.retain(|s| s.monitor != status.monitor);
                let new_len = self.alarms.len();

                if old_len == new_len {
                    Changed::No
                } else {
                    Changed::Yes
                }
            }
            _ => match self.alarms.iter_mut().find(|s| s.monitor == status.monitor) {
                Some(existing) => checked_set(&mut existing.since_millis, status.since_millis)
                    .or(checked_set(&mut existing.state, status.state)),
                None => {
                    unwrap!(self.alarms.push(status));
                    Changed::Yes
                }
            },
        }
    }

    fn overall_state(&self) -> MonitorState {
        let mut state = MonitorState::Normal;

        for a in &self.alarms {
            state = core::cmp::max(state, a.state.clone());
        }

        state
    }
}

pub(crate) static ACTIVE_ALARMS_CHANGED: Watch<CriticalSectionRawMutex, ActiveAlarms, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn monitor_observation_task() {
    let tx = ACTIVE_ALARMS_CHANGED.sender();

    let mut alarms = ActiveAlarms {
        alarms: Default::default(),
    };

    // No alarms at boot time
    tx.send(alarms.clone());

    loop {
        let monitor = NEW_MONITOR_STATUS.receive().await;
        debug!("Monitor changed: {}", monitor);

        queue_telemetry_message(Payload::Process(ProcessPayload::Monitor(monitor.clone()))).await;

        if alarms.update(monitor) == Changed::Yes {
            info!("New alarms: {}", alarms);

            queue_telemetry_message(Payload::Process(ProcessPayload::Alarms(alarms.clone()))).await;

            tx.send(alarms.clone());
        }
    }
}
