use heapless::String;
use serde::{Deserialize, Serialize};

/// An enumeration representing the possible reasons for a system boot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct SystemInformation {
    pub git_revision: String<20>,
    pub last_boot_reason: BootReason,
    pub uptime_milliseconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Severity {
    Normal,
    Information,
    Warning,
    Critical,
}

pub type TemperatureReading = Result<f32, ()>;
