use heapless::Vec;
use serde::{Serialize, de::DeserializeOwned};

pub const RESPONSE_PAYLOAD_CAPACITY: usize = 200;

pub struct Response {
    id: [u8; 4],
    data: Vec<u8, RESPONSE_PAYLOAD_CAPACITY>,
}

impl Response {
    pub fn new<T>(payload: &T) -> Result<Self, ()>
    where
        T: ResponsePayload + Serialize,
    {
        let mut buffer = [0u8; RESPONSE_PAYLOAD_CAPACITY];

        let data = postcard::to_slice_cobs(payload, buffer.as_mut_slice()).map_err(|_| ())?;
        let data = Vec::from_slice(&data).unwrap();

        Ok(Self {
            id: T::id().clone(),
            data,
        })
    }

    pub fn get_payload<T>(&mut self) -> Result<T, ResponseError>
    where
        T: ResponsePayload + DeserializeOwned,
    {
        if self.id != *T::id() {
            return Err(ResponseError::IdMismatch);
        }

        postcard::from_bytes_cobs(&mut self.data).map_err(|_| ResponseError::Deserialize)
    }
}

pub trait ResponsePayload {
    fn id() -> &'static [u8; 4];
}

#[derive(Debug, defmt::Format, PartialEq, Eq)]
pub enum ResponseError {
    IdMismatch,
    Deserialize,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    struct DummyPayload {
        value: u8,
    }

    impl ResponsePayload for DummyPayload {
        fn id() -> &'static [u8; 4] {
            b"test"
        }
    }

    #[test]
    fn round_trip() {
        let payload = DummyPayload { value: 42 };
        let mut response = Response::new(&payload).unwrap();
        let payload_2 = response.get_payload().unwrap();
        assert_eq!(payload, payload_2);
    }

    #[test]
    fn incorrect_id() {
        let payload = DummyPayload { value: 42 };
        let mut response = Response::new(&payload).unwrap();
        response.id = b"arse".to_owned();
        assert!(response.get_payload::<DummyPayload>().is_err());
    }
}
