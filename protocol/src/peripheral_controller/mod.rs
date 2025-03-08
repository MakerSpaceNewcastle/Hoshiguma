use serde::{Deserialize, Serialize};

pub mod stream;
pub mod types;

pub const SERIAL_BAUD: u32 = 9600;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rpc {
    Ping(crate::common_rpc::Ping),
    GetVersion(crate::common_rpc::GetVersion),
    GetUptime(crate::common_rpc::GetUptime),
    Reset(crate::common_rpc::Reset),
    GetEventQueueLength(crate::common_rpc::GetEventQueueLength),
    GetEventMessages(crate::common_rpc::GetEventMessages<self::stream::StreamPayload>),
}
