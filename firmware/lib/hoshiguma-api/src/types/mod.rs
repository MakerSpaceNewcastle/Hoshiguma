mod access_control;
pub use access_control::*;

mod airflow;
pub use airflow::*;

mod onewire_temperature;
pub use onewire_temperature::*;

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
