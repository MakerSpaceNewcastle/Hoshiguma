use crate::TelemetryString;
use core::time::Duration;
use defmt::Format;
use heapless::String;
use serde::{Deserialize, Serialize};

pub type GitRevisionString = String<20>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

impl TelemetryString for BootReason {
    fn telemetry_str(&self) -> &'static str {
        match self {
            BootReason::Normal => "normal",
            BootReason::WatchdogTimeout => "watchdog_timeout",
            BootReason::WatchdogForced => "watchdog_forced",
        }
    }
}

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInformation {
    pub git_revision: GitRevisionString,
    pub uptime: Duration,
    pub boot_reason: BootReason,
}

pub trait SystemInformationRequestPayload {
    fn system_information() -> Self;
}

pub trait SystemInformationResponsePayload {
    fn system_information(self) -> Option<SystemInformation>;
}
