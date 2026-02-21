use heapless::Vec;
use serde::de::DeserializeOwned;

pub const RESPONSE_PAYLOAD_CAPACITY: usize = 200;

pub struct Response {
    id: [u8; 4],
    data: Vec<u8, RESPONSE_PAYLOAD_CAPACITY>,
}

impl Response {
    pub fn get_payload<T>(&mut self) -> Result<T, ()>
    where
        T: ResponsePayload + DeserializeOwned,
    {
        if self.id != *T::id() {
            return Err(());
        }

        let payload = postcard::from_bytes_cobs(&mut self.data).unwrap();

        Ok(payload)
    }
}

pub trait ResponsePayload {
    fn id() -> &'static [u8; 4];
}
