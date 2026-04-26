use crate::{MessageId, MessagePayload};
use core::{net::Ipv4Addr, time::Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetGitRevision,
    GetUptime,
    GetBootReason,

    SubscribeToNotifications(Ipv4Addr),

    SetBacklightMode(crate::HmiBacklightMode),
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

    BacklightMode(crate::HmiBacklightMode),
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Notification {
    AnyPanelInteraction,
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
