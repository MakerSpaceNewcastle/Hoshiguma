pub(crate) mod chassis_intrusion;
pub(crate) mod coolant_level;
pub(crate) mod power;
pub(crate) mod temperatures;

use crate::changed::Changed;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Instant;
use hoshiguma_protocol::payload::process::{Monitor, MonitorState, MonitorStatus};

trait MonitorStateExt {
    fn upgrade(&mut self, other: Self);
}

impl MonitorStateExt for MonitorState {
    fn upgrade(&mut self, other: Self) {
        if other > *self {
            *self = other;
        }
    }
}

trait MonitorStatusExt {
    fn new(monitor: Monitor) -> Self;
    fn refresh(&mut self, state: MonitorState) -> Changed;
}

impl MonitorStatusExt for MonitorStatus {
    fn new(monitor: Monitor) -> Self {
        Self {
            since_millis: Instant::now().as_millis(),
            monitor,
            state: MonitorState::Normal,
        }
    }

    fn refresh(&mut self, state: MonitorState) -> Changed {
        if self.state == state {
            Changed::No
        } else {
            self.since_millis = Instant::now().as_millis();
            self.state = state;
            Changed::Yes
        }
    }
}

pub(crate) static NEW_MONITOR_STATUS: Channel<CriticalSectionRawMutex, MonitorStatus, 16> =
    Channel::new();
