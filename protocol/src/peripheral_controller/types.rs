use crate::types::Severity;
use heapless::LinearMap;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MonitorKind {
    LogicPowerSupplyNotPresent,

    ChassisIntrusion,

    CoolantResevoirLevelSensorFault,
    CoolantResevoirLevel,

    TemperatureSensorFault,
    CoolantFlowTemperature,
    CoolantResevoirTemperature,
}

/// The number of monitors in the system.
///
/// This constant defines the total count of monitor types that are observed.
/// It must be equal to the number of variants of `MonitorKind`.
const NUM_MONITORS: usize = 7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monitors {
    inner: LinearMap<MonitorKind, Severity, NUM_MONITORS>,
}

impl Monitors {
    pub fn get_mut(&mut self, monitor: MonitorKind) -> &mut Severity {
        self.inner.get_mut(&monitor).unwrap()
    }

    pub fn iter(&self) -> heapless::linear_map::Iter<MonitorKind, Severity> {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistDemand {
    Idle,
    Demand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum AirAssistPump {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum FumeExtractionFan {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum LaserEnable {
    Inhibit,
    Enable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineEnable {
    Inhibit,
    Enable,
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

pub type CoolantResevoirLevelReading = Result<CoolantResevoirLevel, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachinePower {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachineRun {
    Idle,
    Running,
}

pub type TemperatureReading = Result<f32, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Temperatures {
    pub onboard: TemperatureReading,
    pub electronics_bay_top: TemperatureReading,

    pub laser_chamber: TemperatureReading,

    pub ambient: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_return: TemperatureReading,

    pub coolant_resevoir_bottom: TemperatureReading,
    pub coolant_resevoir_top: TemperatureReading,

    pub coolant_pump: TemperatureReading,
}
