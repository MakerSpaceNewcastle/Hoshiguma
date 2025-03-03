mod boot;
mod info;
mod panic;

pub use self::{
    boot::{Boot, BootReason},
    info::{GitRevisionString, Info},
    panic::Panic,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum SystemMessagePayload {
    Boot(Boot),
    Heartbeat(Info),
    Panic(Panic),
}
