mod air_assist_pump;
mod fume_extraction_fan;
mod laser_enable;
mod machine_enable;
mod status_lamp;

pub use self::{
    air_assist_pump::AirAssistPump, fume_extraction_fan::FumeExtractionFan,
    laser_enable::LaserEnable, machine_enable::MachineEnable, status_lamp::StatusLamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ControlPayload {
    AirAssistPump(AirAssistPump),
    FumeExtractionFan(FumeExtractionFan),
    LaserEnable(LaserEnable),
    MachineEnable(MachineEnable),
    StatusLamp(StatusLamp),
}
