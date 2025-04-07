use super::types::{
    AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolingDemand, CoolingEnabled,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, Monitors, StatusLamp, Temperatures,
};
use crate::{cooler::types::Temperatures as CoolerTemperatures, types::SystemInformation};
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
    CoolerBoot(SystemInformation),

    MonitorsChanged(Monitors),
    LockoutChanged(MachineOperationLockout),

    CoolingEnableChanged(CoolingEnabled),
    CoolingDemandChanged(CoolingDemand),

    Observation(ObservationEvent),

    Control(ControlEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationEvent {
    AirAssistDemand(AirAssistDemand),
    ChassisIntrusion(ChassisIntrusion),
    FumeExtractionMode(FumeExtractionMode),
    MachinePower(MachinePower),
    MachineRun(MachineRun),
    TemperaturesA(Temperatures),
    TemperaturesB(CoolerTemperatures),
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
