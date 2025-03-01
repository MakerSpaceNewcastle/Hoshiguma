mod air_assist_demand;
mod chassis_intrusion;
mod coolant_resevoir_level;
mod fume_extraction_mode;
mod machine_power;
mod machine_run;
mod temperatures;

pub use self::{
    air_assist_demand::AirAssistDemand,
    chassis_intrusion::ChassisIntrusion,
    coolant_resevoir_level::{CoolantResevoirLevel, CoolantResevoirLevelReading},
    fume_extraction_mode::FumeExtractionMode,
    machine_power::MachinePower,
    machine_run::MachineRunStatus,
    temperatures::Temperatures,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationPayload {
    AirAssistDemand(AirAssistDemand),
    ChassisIntrusion(ChassisIntrusion),
    CoolantResevoirLevel(CoolantResevoirLevelReading),
    FumeExtractionMode(FumeExtractionMode),
    MachinePower(MachinePower),
    MachineRun(MachineRunStatus),
    Temperatures(Temperatures),
}
