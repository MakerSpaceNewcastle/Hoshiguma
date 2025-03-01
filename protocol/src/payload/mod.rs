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
