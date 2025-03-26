use super::types::{
    AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolantResevoirLevelReading,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, Monitors, StatusLamp, Temperatures,
};
use crate::types::SystemInformation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Event {
    pub timestamp_milliseconds: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum EventKind {
    Boot(SystemInformation),
    MonitorsChanged(Monitors),
    LockoutChanged(MachineOperationLockout),
    Observation(ObservationEvent),
    Control(ControlEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationEvent {
    AirAssistDemand(AirAssistDemand),
    ChassisIntrusion(ChassisIntrusion),
    CoolantResevoirLevel(CoolantResevoirLevelReading),
    FumeExtractionMode(FumeExtractionMode),
    MachinePower(MachinePower),
    MachineRun(MachineRun),
    Temperatures(Temperatures),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlEvent {
    AirAssistPump(AirAssistPump),
    FumeExtractionFan(FumeExtractionFan),
    LaserEnable(LaserEnable),
    MachineEnable(MachineEnable),
    StatusLamp(StatusLamp),
}
