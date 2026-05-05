use crate::{MessageId, MessagePayload, SystemInformation, SystemInformationMessage};
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
    SystemInformation(SystemInformation),

    BacklightMode(crate::HmiBacklightMode),
}

impl SystemInformationMessage for ResponseData {
    fn system_information(self) -> Option<SystemInformation> {
        match self {
            ResponseData::SystemInformation(info) => Some(info),
            _ => None,
        }
    }
}
