use super::types::{
    Compressor, CoolantFlow, CoolantPump, HeaderTankCoolantLevelReading, HeatExchangeFluidLevel,
    RadiatorFan, Stirrer, Temperatures,
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
    Control(ControlEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationEvent {
    Temperatures(Temperatures),
    CoolantFlow(CoolantFlow),
    HeatExchangeFluidLevel(HeatExchangeFluidLevel),
    HeaderTankCoolantLevel(HeaderTankCoolantLevelReading),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlEvent {
    Compressor(Compressor),
    RadiatorFan(RadiatorFan),
    Stirrer(Stirrer),
    CoolantPump(CoolantPump),
}
