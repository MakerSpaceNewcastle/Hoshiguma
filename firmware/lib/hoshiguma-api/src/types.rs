use heapless::String;
use serde::{Deserialize, Serialize};

pub type GitRevisionString = String<20>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

pub type TemperatureReading = Result<f32, ()>;
