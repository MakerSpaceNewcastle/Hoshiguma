use super::types::{
    ActiveAlarms, AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolantResevoirLevelReading,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, MonitorStatus, StatusLamp, Temperatures,
};
use crate::types::{BootReason, GitVersionString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Event {
    Boot(BootEvent),
    Observation(ObservationEvent),
    Process(ProcessEvent),
    Control(ControlEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct BootEvent {
    pub git_revision: GitVersionString,
    pub reason: BootReason,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ProcessEvent {
    Monitor(MonitorStatus),
    Alarms(ActiveAlarms),
    Lockout(MachineOperationLockout),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlEvent {
    AirAssistPump(AirAssistPump),
    FumeExtractionFan(FumeExtractionFan),
    LaserEnable(LaserEnable),
    MachineEnable(MachineEnable),
    StatusLamp(StatusLamp),
}
