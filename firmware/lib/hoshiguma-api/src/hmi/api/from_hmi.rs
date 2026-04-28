use crate::{AccessControlRawInput, AccessControlState, MessageId, MessagePayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    NotifyPanelInteraction,
    NotifyAccessControlInputChanged(AccessControlRawInput),
    NotifyAccessControlStateChanged(AccessControlState),
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
    AckPanelInteraction,
    AckAccessControlInputChanged(AccessControlRawInput),
    AckAccessControlStateChanged(AccessControlState),
}
