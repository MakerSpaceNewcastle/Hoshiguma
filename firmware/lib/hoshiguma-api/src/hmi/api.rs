use crate::{MessageId, MessagePayload};
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,
}

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"hmiq"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"hmip"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {
    GitRevision(crate::GitRevisionString),
    Uptime(Duration),
    BootReason(crate::BootReason),
}
