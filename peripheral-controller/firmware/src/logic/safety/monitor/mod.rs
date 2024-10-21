pub(crate) mod chassis_intrusion;
pub(crate) mod coolant_level;
pub(crate) mod power;
pub(crate) mod temperatures;

use crate::changed::Changed;
use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Instant;

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) enum Monitor {
    LogicPowerSupplyNotPresent,

    ChassisIntrusion,

    CoolantResevoirLevelSensorFault,
    CoolantResevoirLevel,

    TemperatureSensorFault,
    CoolantFlowTemperature,
    CoolantResevoirTemperature,
}

#[cfg(feature = "telemetry")]
impl From<&Monitor> for hoshiguma_telemetry_protocol::payload::process::Monitor {
    fn from(value: &Monitor) -> Self {
        match value {
            Monitor::LogicPowerSupplyNotPresent => Self::LogicPowerSupplyNotPresent,
            Monitor::ChassisIntrusion => Self::ChassisIntrusion,
            Monitor::CoolantResevoirLevelSensorFault => Self::CoolantResevoirLevelSensorFault,
            Monitor::CoolantResevoirLevel => Self::CoolantResevoirLevel,
            Monitor::TemperatureSensorFault => Self::TemperatureSensorFault,
            Monitor::CoolantFlowTemperature => Self::CoolantFlowTemperature,
            Monitor::CoolantResevoirTemperature => Self::CoolantResevoirTemperature,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Format)]
pub(crate) enum MonitorState {
    Normal,
    Warn,
    Critical,
}

#[cfg(feature = "telemetry")]
impl From<&MonitorState> for hoshiguma_telemetry_protocol::payload::process::MonitorState {
    fn from(value: &MonitorState) -> Self {
        match value {
            MonitorState::Normal => Self::Normal,
            MonitorState::Warn => Self::Warn,
            MonitorState::Critical => Self::Critical,
        }
    }
}

impl MonitorState {
    pub(crate) fn upgrade(&mut self, other: MonitorState) {
        if other > *self {
            *self = other;
        }
    }
}

#[derive(Clone, Format)]
pub(crate) struct MonitorStatus {
    pub since: Instant,
    pub monitor: Monitor,
    pub state: MonitorState,
}

#[cfg(feature = "telemetry")]
impl From<&MonitorStatus> for hoshiguma_telemetry_protocol::payload::process::MonitorStatus {
    fn from(value: &MonitorStatus) -> Self {
        Self {
            since_millis: value.since.as_millis(),
            monitor: (&value.monitor).into(),
            state: (&value.state).into(),
        }
    }
}

impl MonitorStatus {
    pub(crate) fn new(monitor: Monitor) -> Self {
        Self {
            since: Instant::now(),
            monitor,
            state: MonitorState::Normal,
        }
    }

    pub(crate) fn refresh(&mut self, state: MonitorState) -> Changed {
        if self.state == state {
            Changed::No
        } else {
            self.since = Instant::now();
            self.state = state;
            Changed::Yes
        }
    }
}

pub(crate) static NEW_MONITOR_STATUS: Channel<CriticalSectionRawMutex, MonitorStatus, 16> =
    Channel::new();
