use super::types::GitVersionString;
use crate::RpcMessage;

pub type Ping = RpcMessage<u32, u32>;
pub type GetVersion = RpcMessage<(), GitVersionString>;
pub type GetUptime = RpcMessage<(), u64>;
pub type Reset = RpcMessage<(), ()>;
