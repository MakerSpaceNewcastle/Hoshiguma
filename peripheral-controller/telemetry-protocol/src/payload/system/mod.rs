mod boot;
mod panic;

pub use self::{boot::Boot, panic::Panic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SystemMessagePayload {
    Boot(Boot),
    Panic(Panic),
}
