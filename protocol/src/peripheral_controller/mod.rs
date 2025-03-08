pub mod stream;
pub mod types;

use serde::{Deserialize, Serialize};

pub type ControllerMessage = super::Message<Rpc, stream::StreamPayload>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rpc {
    Ping(crate::common::Ping),
    GetVersion(crate::common::GetVersion),
    GetUptime(crate::common::GetUptime),
    Reset(crate::common::Reset),
}
