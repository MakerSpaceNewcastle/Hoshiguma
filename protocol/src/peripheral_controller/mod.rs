pub mod stream;
pub mod types;

use serde::{Deserialize, Serialize};

pub type ControllerMessage = super::Message<Rpc, stream::StreamPayload>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rpc {
    Ping(crate::common::rpc::Ping),
    GetVersion(crate::common::rpc::GetVersion),
    GetUptime(crate::common::rpc::GetUptime),
    Reset(crate::common::rpc::Reset),
}
