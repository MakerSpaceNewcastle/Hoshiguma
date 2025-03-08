use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rpc {
    Ping(crate::common::rpc::Ping),
    GetVersion(crate::common::rpc::GetVersion),
    GetUptime(crate::common::rpc::GetUptime),
    Reset(crate::common::rpc::Reset),
}
