use defmt::Format;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, EnumString};

#[derive(
    Debug,
    Format,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Display,
    EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum Severity {
    Normal,
    Information,
    Warning,
    Critical,
    Fatal,
}

#[derive(
    Debug,
    Format,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum Monitor {
    /// A fault occurred that will not allow the machine to be powered back on normally
    InterlockTripped,

    /// Is there power on the switched mains AC bus?
    AcBusPower,

    /// Are all the interlocked doors closed?
    Doors,

    /// Is there active communication with the cooler?
    CoolerCommunication,

    /// Is there active communication with the rear sensor board?
    RearSensorBoardCommunication,

    /// Is there active communication with the HMI?
    HmiCommunication,

    /// Is there active communication with the telemetry bridge?
    TelemetryBridgeCommunication,

    /// Is the rate of coolant flow and return equal within limits?
    CoolantRateSymmetry,

    /// Is the rate of coolant circulation sufficient?
    CoolantRate,

    /// Are all temperature sensors reporting?
    TemperatureSensorsFunctional,

    /// Are the electronics (of any part of the system) at a suitable temperature?
    ElectronicsTemperature,

    /// Is the temperature of the coolant in the flow line within limits?
    CoolantFlowTemperature,

    /// Is the temperature of the coolant in the reservoir within limits?
    CoolantReservoirTemperature,

    /// Is the fume extraction airflow sufficient?
    ExtractionAirflow,

    /// Is the fume extraction airflow sensor reporting correctly?
    ExtractionAirflowSensorFunctional,
}
