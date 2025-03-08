use serde::{Deserialize, Serialize};

use super::types::{
    ActiveAlarms, AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolantResevoirLevelReading,
    FumeExtractionFan, FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout,
    MachinePower, MachineRun, MonitorStatus, StatusLamp, Temperatures,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum StreamPayload {
    // System(SystemMessagePayload),
    Observation(ObservationPayload),
    Process(ProcessPayload),
    Control(ControlPayload),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationPayload {
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
pub enum ProcessPayload {
    Monitor(MonitorStatus),
    Alarms(ActiveAlarms),
    Lockout(MachineOperationLockout),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlPayload {
    AirAssistPump(AirAssistPump),
    FumeExtractionFan(FumeExtractionFan),
    LaserEnable(LaserEnable),
    MachineEnable(MachineEnable),
    StatusLamp(StatusLamp),
}
