use heapless::Vec;

pub const RESPONSE_PAYLOAD_CAPACITY: usize = 200;

pub struct Response {
    id: [u8; 4],
    data: Vec<u8, RESPONSE_PAYLOAD_CAPACITY>,
}
