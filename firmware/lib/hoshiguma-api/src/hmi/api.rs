use crate::{MessageId, MessagePayload};
use core::{net::Ipv4Addr, time::Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,

    SubscribeToNotifications(Ipv4Addr),
}

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"hmiQ"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"hmiP"
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
    AccessControlStateChanged {
        raw: crate::AccessControlRawInput,
        state: crate::AccessControlState,
    },
}

impl MessagePayload for Notification {
    fn id() -> &'static MessageId {
        b"hmiN"
    }
}
