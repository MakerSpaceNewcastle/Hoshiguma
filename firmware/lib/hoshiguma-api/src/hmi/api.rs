use crate::{MessageId, MessagePayload};
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,

    SubscribeToNotifications,
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

    SubscribedToNotifications,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Notification {
    Todo,
}

impl MessagePayload for Notification {
    fn id() -> &'static MessageId {
        b"rsbn"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Acknowledgement;

impl MessagePayload for Acknowledgement {
    fn id() -> &'static MessageId {
        b"rsba"
    }
}
