pub mod control;
pub mod observation;
pub mod process;
pub mod system;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Payload {
    System(system::SystemMessagePayload),
    Observation(observation::ObservationPayload),
    Process(process::ProcessPayload),
    Control(control::ControlPayload),
}

pub enum Message {
    Rpc(cooler::Rpc),
    Publish,
}

pub enum RpcMessage<REQ, RES> {
    Request(REQ),
    Response(RES),
}

mod cooler {
    pub enum Rpc {
        Ping(super::RpcMessage<u32, u32>),
        GetVersion(super::RpcMessage<(), crate::TelemString>),
    }
}
