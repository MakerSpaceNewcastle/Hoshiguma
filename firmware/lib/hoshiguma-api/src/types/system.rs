use core::time::Duration;
use defmt::Format;
use heapless::String;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub type GitRevisionString = String<20>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInformation {
    pub git_revision: GitRevisionString,
    pub uptime: Duration,
    pub boot_reason: BootReason,
}
