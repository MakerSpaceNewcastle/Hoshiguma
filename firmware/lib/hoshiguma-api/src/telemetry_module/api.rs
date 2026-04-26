use crate::{MessageId, MessagePayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {}

impl MessagePayload for Request {
    fn id() -> &'static MessageId {
        b"tlmQ"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response(pub Result<ResponseData, ()>);

impl MessagePayload for Response {
    fn id() -> &'static MessageId {
        b"tlmP"
    }
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseData {}
