use crate::payload::process::MonitorStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveAlarms {
    pub alarms: crate::Vec<MonitorStatus, 16>,
}
