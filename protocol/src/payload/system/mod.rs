mod info;
mod panic;

pub use self::{info::Info, panic::Panic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum SystemMessagePayload {
    Boot(Info),
    Heartbeat(Info),
    Panic(Panic),
}
