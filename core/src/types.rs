use crate::telemetry::{AsTelemetry, StaticTelemetryDataPoint, TelemetryStrValue, TelemetryValue};
use heapless::{LinearMap, String, Vec};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

pub type GitRevisionString = String<20>;

/// An enumeration representing the possible reasons for a system boot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum BootReason {
    Normal,
    WatchdogTimeout,
    WatchdogForced,
}

impl TelemetryStrValue for BootReason {
    fn telemetry_str(&self) -> &'static str {
        match self {
            BootReason::Normal => "normal",
            BootReason::WatchdogTimeout => "watchdog_timeout",
            BootReason::WatchdogForced => "watchdog_forced",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Severity {
    Normal,
    Information,
    Warning,
    Critical,
}

impl TelemetryStrValue for Severity {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Information => "information",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MonitorKind {
    MachinePowerOff,
    ChassisIntrusion,
    CoolerCommunicationFault,
    MachineElectronicsOvertemperature,
    CoolerElectronicsOvertemperature,
    CoolantReservoirLevelLow,
    CoolantFlowInsufficient,
    TemperatureSensorFaultA,
    TemperatureSensorFaultB,
    CoolantFlowOvertemperature,
    CoolantReservoirOvertemperature,
    ExtractionAirflowInsufficient,
    ExtractionAirflowSensorFault,
    TelemetryInop,
}

impl TelemetryStrValue for MonitorKind {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::MachinePowerOff => "monitor_state,monitor=ac_bus_off",
            Self::ChassisIntrusion => "monitor_state,monitor=door_open",
            Self::CoolerCommunicationFault => "monitor_state,monitor=cooler_fault",
            Self::MachineElectronicsOvertemperature => {
                "monitor_state,monitor=machine_electronics_temperature"
            }
            Self::CoolerElectronicsOvertemperature => {
                "monitor_state,monitor=cooler_electronics_temperature"
            }
            Self::CoolantReservoirLevelLow => "monitor_state,monitor=coolant_reservoir_level",
            Self::CoolantFlowInsufficient => "monitor_state,monitor=coolant_flow",
            Self::TemperatureSensorFaultA => "monitor_state,monitor=temperature_sensor_fault_bus_a",
            Self::TemperatureSensorFaultB => "monitor_state,monitor=temperature_sensor_fault_bus_b",
            Self::CoolantFlowOvertemperature => "monitor_state,monitor=flow_coolant_temperature",
            Self::CoolantReservoirOvertemperature => {
                "monitor_state,monitor=coolant_reservoir_temperature"
            }
            Self::ExtractionAirflowInsufficient => "monitor_state,monitor=extraction_airflow",
            Self::ExtractionAirflowSensorFault => {
                "monitor_state,monitor=extraction_airflow_sensor_fault"
            }
            Self::TelemetryInop => "monitor_state,monitor=telemetry_inop",
        }
    }
}

/// The number of monitors in the system.
///
/// This constant defines the total count of monitor types that are observed.
/// It must be equal to the number of variants of `MonitorKind`.
const NUM_MONITORS: usize = 14;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monitors {
    inner: LinearMap<MonitorKind, Severity, NUM_MONITORS>,
}

impl Monitors {
    pub fn get(&self, monitor: MonitorKind) -> &Severity {
        self.inner.get(&monitor).unwrap()
    }

    pub fn get_mut(&mut self, monitor: MonitorKind) -> &mut Severity {
        self.inner.get_mut(&monitor).unwrap()
    }

    pub fn iter(&self) -> heapless::linear_map::Iter<'_, MonitorKind, Severity> {
        self.inner.iter()
    }

    pub fn severity(&self) -> Severity {
        let mut severity = Severity::Normal;
        for s in self.inner.values() {
            severity = core::cmp::max(severity, s.clone());
        }
        severity
    }
}

#[cfg(feature = "no-std")]
impl defmt::Format for Monitors {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(fmt, "Monitors(");
        for (i, (monitor, severity)) in self.inner.iter().enumerate() {
            defmt::write!(fmt, "{} = {}", monitor, severity);
            if i < self.inner.len() - 1 {
                defmt::write!(fmt, ", ");
            }
        }
        defmt::write!(fmt, ")");
    }
}

impl Default for Monitors {
    fn default() -> Self {
        // Default to all monitors being critical until updated otherwise.
        let mut inner = LinearMap::new();
        for m in MonitorKind::iter() {
            inner.insert(m, Severity::Critical).unwrap();
        }
        Self { inner }
    }
}

impl AsTelemetry<{ NUM_MONITORS + 5 }, { NUM_MONITORS }> for Monitors {
    fn strings() -> [&'static str; NUM_MONITORS + 5] {
        [
            MonitorKind::MachinePowerOff.telemetry_str(),
            MonitorKind::ChassisIntrusion.telemetry_str(),
            MonitorKind::CoolerCommunicationFault.telemetry_str(),
            MonitorKind::MachineElectronicsOvertemperature.telemetry_str(),
            MonitorKind::CoolerElectronicsOvertemperature.telemetry_str(),
            MonitorKind::CoolantReservoirLevelLow.telemetry_str(),
            MonitorKind::CoolantFlowInsufficient.telemetry_str(),
            MonitorKind::TemperatureSensorFaultA.telemetry_str(),
            MonitorKind::TemperatureSensorFaultB.telemetry_str(),
            MonitorKind::CoolantFlowOvertemperature.telemetry_str(),
            MonitorKind::CoolantReservoirOvertemperature.telemetry_str(),
            MonitorKind::ExtractionAirflowInsufficient.telemetry_str(),
            MonitorKind::ExtractionAirflowSensorFault.telemetry_str(),
            MonitorKind::TelemetryInop.telemetry_str(),
            "value",
            Severity::Normal.telemetry_str(),
            Severity::Information.telemetry_str(),
            Severity::Warning.telemetry_str(),
            Severity::Critical.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, { NUM_MONITORS }> {
        let mut v = Vec::new();

        for (kind, state) in self.iter() {
            v.push(StaticTelemetryDataPoint {
                measurement: kind.telemetry_str(),
                field: "value",
                value: TelemetryValue::StaticString(state.telemetry_str()),
                timestamp_nanoseconds: None,
            })
            .unwrap();
        }

        v
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}

impl AsTelemetry<5, 1> for MachineOperationLockout {
    fn strings() -> [&'static str; 5] {
        [
            "machine_lockout",
            "value",
            Self::Permitted.telemetry_str(),
            Self::PermittedUntilIdle.telemetry_str(),
            Self::Denied.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "machine_lockout",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for MachineOperationLockout {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Permitted => "permitted",
            Self::PermittedUntilIdle => "permitted_until_idle",
            Self::Denied => "denied",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolingEnable {
    Inhibit,
    Enable,
}

impl AsTelemetry<4, 1> for CoolingEnable {
    fn strings() -> [&'static str; 4] {
        [
            "cooling_enable",
            "value",
            Self::Inhibit.telemetry_str(),
            Self::Enable.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "cooling_enable",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for CoolingEnable {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Inhibit => "inhibit",
            Self::Enable => "enable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolingDemand {
    Idle,
    Demand,
}

impl AsTelemetry<4, 1> for CoolingDemand {
    fn strings() -> [&'static str; 4] {
        [
            "cooling_demand",
            "value",
            Self::Idle.telemetry_str(),
            Self::Demand.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "cooling_demand",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for CoolingDemand {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Demand => "demand",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistDemand {
    Idle,
    Demand,
}

impl AsTelemetry<4, 1> for AirAssistDemand {
    fn strings() -> [&'static str; 4] {
        [
            "air_assist_demand",
            "value",
            Self::Idle.telemetry_str(),
            Self::Demand.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "air_assist_demand",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for AirAssistDemand {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Demand => "demand",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistPump {
    Idle,
    Run,
}

impl AsTelemetry<4, 1> for AirAssistPump {
    fn strings() -> [&'static str; 4] {
        [
            "air_assist_pump",
            "value",
            Self::Idle.telemetry_str(),
            Self::Run.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "air_assist_pump",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for AirAssistPump {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

impl AsTelemetry<4, 1> for FumeExtractionMode {
    fn strings() -> [&'static str; 4] {
        [
            "fume_extraction_mode",
            "value",
            Self::Automatic.telemetry_str(),
            Self::OverrideRun.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "fume_extraction_mode",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for FumeExtractionMode {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::OverrideRun => "override_run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionFan {
    Idle,
    Run,
}

impl AsTelemetry<4, 1> for FumeExtractionFan {
    fn strings() -> [&'static str; 4] {
        [
            "fume_extraction_fan",
            "value",
            Self::Idle.telemetry_str(),
            Self::Run.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "fume_extraction_fan",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for FumeExtractionFan {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum LaserEnable {
    Inhibit,
    Enable,
}

impl AsTelemetry<4, 1> for LaserEnable {
    fn strings() -> [&'static str; 4] {
        [
            "laser_enable",
            "value",
            Self::Inhibit.telemetry_str(),
            Self::Enable.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "laser_enable",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for LaserEnable {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Inhibit => "inhibit",
            Self::Enable => "enable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineEnable {
    Inhibit,
    Enable,
}

impl AsTelemetry<4, 1> for MachineEnable {
    fn strings() -> [&'static str; 4] {
        [
            "machine_enable",
            "value",
            Self::Inhibit.telemetry_str(),
            Self::Enable.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "machine_enable",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for MachineEnable {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Inhibit => "inhibit",
            Self::Enable => "enable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct StatusLamp {
    pub red: bool,
    pub amber: bool,
    pub green: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum ChassisIntrusion {
    Normal,
    Intruded,
}

impl AsTelemetry<4, 1> for ChassisIntrusion {
    fn strings() -> [&'static str; 4] {
        [
            "chassis_intrusion",
            "value",
            Self::Normal.telemetry_str(),
            Self::Intruded.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "chassis_intrusion",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for ChassisIntrusion {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Intruded => "intruded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachinePower {
    On,
    Off,
}

impl AsTelemetry<4, 1> for MachinePower {
    fn strings() -> [&'static str; 4] {
        [
            "machine_power",
            "value",
            Self::On.telemetry_str(),
            Self::Off.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "machine_power",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for MachinePower {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineRun {
    Idle,
    Running,
}

impl AsTelemetry<4, 1> for MachineRun {
    fn strings() -> [&'static str; 4] {
        [
            "machine_run",
            "value",
            Self::Idle.telemetry_str(),
            Self::Running.telemetry_str(),
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 1> {
        Vec::from_array([StaticTelemetryDataPoint {
            measurement: "machine_run",
            field: "value",
            value: TelemetryValue::StaticString(self.telemetry_str()),
            timestamp_nanoseconds: None,
        }])
    }
}

impl TelemetryStrValue for MachineRun {
    fn telemetry_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
        }
    }
}

pub type TemperatureReading = Result<f32, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct MachineTemperatures {
    pub onboard: TemperatureReading,
    pub electronics_bay_top: TemperatureReading,

    pub laser_chamber: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_return: TemperatureReading,
}

impl AsTelemetry<7, 10> for MachineTemperatures {
    fn strings() -> [&'static str; 7] {
        [
            "temperature,sensor=controller_onboard",
            "temperature,sensor=electronics_bay",
            "temperature,sensor=laser_chamber",
            "temperature,sensor=coolant_flow",
            "temperature,sensor=coolant_return",
            "value",
            "sensor_ok",
        ]
    }

    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, 10> {
        let mut v = Vec::new();

        for (name, reading) in [
            ("temperature,sensor=controller_onboard", self.onboard),
            (
                "temperature,sensor=electronics_bay",
                self.electronics_bay_top,
            ),
            ("temperature,sensor=laser_chamber", self.laser_chamber),
            ("temperature,sensor=coolant_flow", self.coolant_flow),
            ("temperature,sensor=coolant_return", self.coolant_return),
        ] {
            v.push(StaticTelemetryDataPoint {
                measurement: name,
                field: "sensor_ok",
                value: TelemetryValue::Bool(reading.is_ok()),
                timestamp_nanoseconds: None,
            })
            .unwrap();

            if let Ok(reading) = reading {
                v.push(StaticTelemetryDataPoint {
                    measurement: name,
                    field: "value",
                    value: TelemetryValue::Float32(reading),
                    timestamp_nanoseconds: None,
                })
                .unwrap();
            }
        }

        v
    }
}
