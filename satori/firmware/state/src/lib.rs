pub mod sensors;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalMachineState {
    koishi_io: KoishiIo,
    coolant_level: sensors::CoolantLevelReading,
    // TODO
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KoishiIo {
    inputs: koishi_telemetry_protocol::Inputs,
    outputs: koishi_telemetry_protocol::Outputs,
}
