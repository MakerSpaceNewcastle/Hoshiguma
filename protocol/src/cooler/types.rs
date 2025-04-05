use crate::types::{Severity, TemperatureReading};
use heapless::LinearMap;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MonitorKind {
    CoolantHeaderTankLevelSensorFault,
    CoolantHeaderTankEmpty,
    CoolantHeaderTankOverfilled,

    HeatExchangerFluidLow,

    HeatExchangerOvertemperature,
    CoolantFlowOvertemperature,

    CoolantFlowInsufficient,
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
pub struct Temperatures {
    pub onboard: TemperatureReading,

    pub coolant_flow: TemperatureReading,
    pub coolant_mid: TemperatureReading,
    pub coolant_return: TemperatureReading,

    pub heat_exchange_fluid: TemperatureReading,
    pub heat_exchanger_loop: TemperatureReading,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Compressor {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum RadiatorFan {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Stirrer {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantPump {
    Idle,
    Run,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum HeatExchangeFluidLevel {
    Normal,
    Low,
}

pub type HeaderTankCoolantLevelReading = Result<HeaderTankCoolantLevel, ()>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum HeaderTankCoolantLevel {
    Empty,
    Normal,
    Full,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct CoolantFlow(f64);

impl CoolantFlow {
    pub fn new(litres: f64, seconds: f64) -> Self {
        Self(litres / seconds)
    }
}
