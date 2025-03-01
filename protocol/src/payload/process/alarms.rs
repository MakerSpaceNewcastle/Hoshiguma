use crate::payload::process::MonitorStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct ActiveAlarms {
    #[cfg(feature = "std")]
    pub alarms: std::vec::Vec<MonitorStatus>,
    #[cfg(not(feature = "std"))]
    pub alarms: heapless::Vec<MonitorStatus, 16>,
}
