pub mod control;
pub mod observation;
pub mod process;
pub mod system;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Payload {
    System(system::SystemMessagePayload),
    Observation(observation::ObservationPayload),
    Process(process::ProcessPayload),
    Control(control::ControlPayload),
}
