use super::types::{
    Compressor, CoolantPump, HeatExchangeFluidLevel, RadiatorFan, Stirrer, Temperatures,
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
    // TODO
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ObservationEvent {
    Temperatures(Temperatures),
    HeatExchangeFluidLevel(HeatExchangeFluidLevel),
    // TODO
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ControlEvent {
    Compressor(Compressor),
    RadiatorFan(RadiatorFan),
    Stirrer(Stirrer),
    CoolantPump(CoolantPump),
}
