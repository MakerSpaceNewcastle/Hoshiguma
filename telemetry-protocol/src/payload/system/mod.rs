mod info;
mod panic;

pub use self::{info::Info, panic::Panic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SystemMessagePayload {
    Boot(Info),
    Heartbeat(Info),
    Panic(Panic),
}
