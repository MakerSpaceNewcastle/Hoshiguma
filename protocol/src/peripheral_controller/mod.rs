pub mod rpc;
pub mod stream;
pub mod types;

pub const SERIAL_BAUD: u32 = 9600;

pub type ControllerMessage = super::Message<rpc::Rpc, stream::StreamPayload>;
