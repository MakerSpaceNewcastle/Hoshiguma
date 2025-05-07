use super::types::{
    AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolingDemand, CoolingEnabled,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, Monitors, StatusLamp, Temperatures,
};
use crate::{
    cooler::types::{
        CompressorState, CoolantFlow, CoolantPumpState, CoolantReservoirLevel, RadiatorFanState,
        Temperatures as CoolerTemperatures,
    },
    types::SystemInformation,
};
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
    // Self
    TemperaturesA(Temperatures),
    AirAssistDemand(AirAssistDemand),
    ChassisIntrusion(ChassisIntrusion),
    FumeExtractionMode(FumeExtractionMode),
    MachinePower(MachinePower),
    MachineRun(MachineRun),

    // Forwarded from cooler
    TemperaturesB(CoolerTemperatures),
    CoolantFlow(CoolantFlow),
    CoolantReservoirLevel(CoolantReservoirLevel),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlEvent {
    // Self
    AirAssistPump(AirAssistPump),
    FumeExtractionFan(FumeExtractionFan),
    LaserEnable(LaserEnable),
    MachineEnable(MachineEnable),
    StatusLamp(StatusLamp),

    // Forwarded from cooler
    CoolerCompressor(CompressorState),
    CoolerRadiatorFan(RadiatorFanState),
    CoolantPump(CoolantPumpState),
}
