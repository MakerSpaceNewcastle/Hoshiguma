use defmt::Format;
use heapless::LinearMap;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Format, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Normal,
    Information,
    Warning,
    Critical,
}

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
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
