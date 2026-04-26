use defmt::Format;
use heapless::Vec;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const MAX_MESSAGE_SIZE: usize = MESSAGE_PAYLOAD_CAPACITY + 10;
pub const MESSAGE_PAYLOAD_CAPACITY: usize = 512;

pub type MessageId = [u8; 4];

#[derive(Debug, Format, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    id: MessageId,
    data: Vec<u8, MESSAGE_PAYLOAD_CAPACITY>,
}

impl Message {
    pub fn new<T>(payload: &T) -> Result<Self, postcard::Error>
    where
        T: MessagePayload + Serialize,
    {
        let mut buffer = [0u8; MESSAGE_PAYLOAD_CAPACITY];

        let data = postcard::to_slice(payload, buffer.as_mut_slice())?;
        let data = Vec::from_slice(data).unwrap();

        Ok(Self { id: *T::id(), data })
    }

    pub fn from_bytes(bytes: &mut [u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes_cobs(bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8, MAX_MESSAGE_SIZE>, postcard::Error> {
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        let data = postcard::to_slice_cobs(self, buffer.as_mut_slice())?;
        Ok(Vec::from_slice(data).unwrap())
    }

    pub fn id(&self) -> MessageId {
        self.id
    }

    pub fn payload<T>(&mut self) -> Result<T, MessageError>
    where
        T: MessagePayload + DeserializeOwned,
    {
        if self.id != *T::id() {
            return Err(MessageError::IdMismatch);
        }

        postcard::from_bytes(&self.data).map_err(MessageError::Deserialize)
    }
}

pub trait MessagePayload {
    fn id() -> &'static MessageId;
}

#[derive(Debug, PartialEq, Eq)]
pub enum MessageError {
    IdMismatch,
    Deserialize(postcard::Error),
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    struct DummyPayload {
        value: u8,
    }

    impl MessagePayload for DummyPayload {
        fn id() -> &'static MessageId {
            b"test"
        }
    }

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    struct OtherDummyPayload {
        value: u8,
    }

    impl MessagePayload for OtherDummyPayload {
        fn id() -> &'static MessageId {
            b"toot"
        }
    }

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    struct ChonkPayload {
        value: Vec<u8, MESSAGE_PAYLOAD_CAPACITY>,
    }

    impl MessagePayload for ChonkPayload {
        fn id() -> &'static MessageId {
            b"bigg"
        }
    }

    #[test]
    fn round_trip() {
        let payload_tx = DummyPayload { value: 42 };

        let message_tx = Message::new(&payload_tx).unwrap();
        assert_eq!(message_tx.data.len(), 1);

        let mut bytes = message_tx.to_bytes().unwrap();
        assert_eq!(bytes.len(), 8);

        let mut message_rx = Message::from_bytes(&mut bytes).unwrap();
        let payload_rx = message_rx.payload().unwrap();

        assert_eq!(payload_tx, payload_rx);
    }

    #[test]
    fn get_payload_multiple_times() {
        let payload = DummyPayload { value: 42 };
        let mut message = Message::new(&payload).unwrap();
        assert_eq!(payload, message.payload().unwrap());
        assert_eq!(payload, message.payload().unwrap());
    }

    #[test]
    fn get_payload_after_fail() {
        let payload = DummyPayload { value: 42 };
        let mut message = Message::new(&payload).unwrap();
        assert!(message.payload::<OtherDummyPayload>().is_err());
        assert_eq!(payload, message.payload().unwrap());
    }

    #[test]
    fn incorrect_id() {
        let payload = DummyPayload { value: 42 };
        let mut message = Message::new(&payload).unwrap();
        assert_eq!(
            message.payload::<OtherDummyPayload>(),
            Err(MessageError::IdMismatch)
        );
    }

    #[test]
    fn size() {
        let payload = ChonkPayload {
            value: Vec::from_array([42u8; MESSAGE_PAYLOAD_CAPACITY - 2]),
        };
        let message = Message::new(&payload).unwrap();
        assert_eq!(message.data.len(), MESSAGE_PAYLOAD_CAPACITY);

        let bytes = message.to_bytes().unwrap();
        assert_eq!(bytes.len(), MAX_MESSAGE_SIZE);
    }
}
