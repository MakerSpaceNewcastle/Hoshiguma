use super::GitRevisionString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Boot {
    pub git_revision: GitRevisionString,
    pub reason: BootReason,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}
