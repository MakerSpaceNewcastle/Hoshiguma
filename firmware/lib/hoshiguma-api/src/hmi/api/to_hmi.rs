use crate::{MessageId, MessagePayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    GetSystemInformation,

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
    SystemInformation(crate::SystemInformation),

    BacklightMode(crate::HmiBacklightMode),
}
