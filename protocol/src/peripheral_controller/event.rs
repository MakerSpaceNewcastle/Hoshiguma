use super::types::{
    ActiveAlarms, AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolantResevoirLevelReading,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, MonitorStatus, StatusLamp, Temperatures,
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
    Observation(ObservationEvent),
    Process(ProcessEvent),
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ProcessEvent {
    Monitor(MonitorStatus),
    Alarms(ActiveAlarms),
    Lockout(MachineOperationLockout),
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
