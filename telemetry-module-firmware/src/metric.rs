use crate::network::{telemetry_tx::BUFFER_FREE_SPACE_THRESHOLD, LinkState};
use core::{fmt::Write, time::Duration};
use defmt::Format;
use heapless::String;
use hoshiguma_protocol::{
    peripheral_controller::{
        event::{ControlEvent, EventKind, ObservationEvent},
        types::MonitorKind,
    },
    types::{SystemInformation, TemperatureReading},
};

#[derive(Clone)]
pub(crate) struct Metric {
    timestamp: Option<Duration>,
    value: MetricKind,
}

#[derive(Clone)]
pub(crate) enum MetricKind {
    TelemetryModuleSystemInformation(SystemInformation),
    TelemetryModuleNetworkState(LinkState),
    TelemetryModuleTime(TimeMetrics),
    TelemetryModuleStatistics(Statistics),
    PeripheralControllerEvent(EventKind),
}

#[derive(Clone, Format)]
pub(crate) struct TimeMetrics {
    pub(crate) unix_epoch_nanoseconds: u64,
    pub(crate) sync_age_nanoseconds: u64,
}

#[derive(Clone, Format)]
pub(crate) struct Statistics {
    pub(crate) events_received: u64,
    pub(crate) receive_failures: u64,

    pub(crate) metrics_added_to_transmit_buffer: u64,
    pub(crate) messages_transmitted: u64,
    pub(crate) tramsit_buffer_failures: u64,
    pub(crate) transmit_network_failures: u64,
}

type TimestampString = String<32>;
type MetricBuildBufferString = String<BUFFER_FREE_SPACE_THRESHOLD>;

fn write_measurement_str_debug(
    s: &mut MetricBuildBufferString,
    measurement: &str,
    value: impl core::fmt::Debug,
    timestamp: TimestampString,
) -> Result<(), core::fmt::Error> {
    s.write_fmt(format_args!("{measurement} value=\"{value:?}\"{timestamp}"))
}

fn write_measurement_numerical(
    s: &mut MetricBuildBufferString,
    measurement: &str,
    unit: &str,
    value: impl core::fmt::Display,
    timestamp: TimestampString,
) -> Result<(), core::fmt::Error> {
    s.write_fmt(format_args!(
        "{measurement},unit={unit} value={value}{timestamp}"
    ))
}

fn write_temperature_sensors(
    s: &mut MetricBuildBufferString,
    source: &str,
    readings: &[(&str, TemperatureReading)],
    timestamp: TimestampString,
) -> Result<(), core::fmt::Error> {
    s.write_fmt(format_args!(
        "observation.temperature,unit=C,source={source} "
    ))?;

    let mut any_previous_fields = false;

    for (name, value) in readings {
        if let Ok(value) = value {
            if any_previous_fields {
                s.write_char(',')?;
            }

            s.write_fmt(format_args!("{name}={value}"))?;
            any_previous_fields = true;
        }
    }

    // Fail if no sensors have values
    if !any_previous_fields {
        return Err(core::fmt::Error);
    }

    s.write_str(&timestamp)
}

impl Metric {
    pub(crate) fn new(timestamp: Option<Duration>, value: MetricKind) -> Self {
        Self { timestamp, value }
    }

    pub(crate) fn format_influx<const N: usize>(
        &self,
        buffer: &mut String<N>,
    ) -> Result<(), core::fmt::Error> {
        let timestamp = {
            let mut s = TimestampString::new();

            match self.timestamp {
                Some(timestamp) => {
                    s.write_fmt(format_args!(" {}\n", timestamp.as_nanos()))?;
                }
                None => {
                    s.write_char('\n')?;
                }
            }

            s
        };

        let mut s = MetricBuildBufferString::new();

        match &self.value {
            MetricKind::TelemetryModuleSystemInformation(v) => {
                s.write_fmt(format_args!(
                    "telemetry_module.system_info git_revision=\"{}\",boot_reason=\"{:?}\"{}",
                    v.git_revision, v.last_boot_reason, timestamp
                ))?;
                write_measurement_numerical(
                    &mut s,
                    "telemetry_module.time.uptime",
                    "ms",
                    v.uptime_milliseconds,
                    timestamp,
                )?;
            }
            MetricKind::TelemetryModuleNetworkState(v) => {
                write_measurement_numerical(
                    &mut s,
                    "telemetry_module.network.connection_age",
                    "s",
                    v.age().as_secs(),
                    timestamp,
                )?;
            }
            MetricKind::TelemetryModuleTime(v) => {
                s.write_fmt(format_args!(
                    "telemetry_module.time.wall,unit=ns unix_epoch={},sync_age={}{}",
                    v.unix_epoch_nanoseconds, v.sync_age_nanoseconds, timestamp
                ))?;
            }
            MetricKind::TelemetryModuleStatistics(v) => {
                s.write_fmt(format_args!(
                    "telemetry_module.statistics.receive events={},failures={}{}",
                    v.events_received, v.receive_failures, timestamp
                ))?;
                s.write_fmt(format_args!(
                    "telemetry_module.statistics.transmit buffer_submissions={},messages={},buffer_failures={},network_failures={}{}",
                    v.metrics_added_to_transmit_buffer,
                    v.messages_transmitted,
                    v.tramsit_buffer_failures,
                    v.transmit_network_failures,
                    timestamp
                ))?;
            }
            MetricKind::PeripheralControllerEvent(event) => match event {
                EventKind::Boot(event) => {
                    s.write_fmt(format_args!(
                            "peripheral_controller_system_info git_revision=\"{}\",boot_reason=\"{:?}\"{}",
                            event.git_revision, event.last_boot_reason, timestamp
                        ))?;
                }
                EventKind::CoolerBoot(event) => {
                    s.write_fmt(format_args!(
                        "cooler_system_info git_revision=\"{}\",boot_reason=\"{:?}\"{}",
                        event.git_revision, event.last_boot_reason, timestamp
                    ))?;
                }
                EventKind::MonitorsChanged(v) => {
                    s.write_str("safety.monitors ")?;

                    for (i, (name, value)) in [
                        ("mach_power_off", MonitorKind::MachinePowerOff),
                        ("chassis_intrusion", MonitorKind::ChassisIntrusion),
                        ("cooler_comm_fault", MonitorKind::CoolerCommunicationFault),
                        (
                            "mach_elec_temp",
                            MonitorKind::MachineElectronicsOvertemperature,
                        ),
                        (
                            "cool_elec_temp",
                            MonitorKind::CoolerElectronicsOvertemperature,
                        ),
                        (
                            "coolant_reservoir_level",
                            MonitorKind::CoolantReservoirLevelLow,
                        ),
                        ("cool_flow_rate", MonitorKind::CoolantFlowInsufficient),
                        ("temp_sens_a", MonitorKind::TemperatureSensorFaultA),
                        ("temp_sens_b", MonitorKind::TemperatureSensorFaultB),
                        ("cool_flow_temp", MonitorKind::CoolantFlowOvertemperature),
                        (
                            "coolant_reservoir_temp",
                            MonitorKind::CoolantReservoirOvertemperature,
                        ),
                    ]
                    .into_iter()
                    .map(|(name, kind)| (name, v.get(kind)))
                    .enumerate()
                    {
                        if i > 0 {
                            s.write_char(',')?;
                        }
                        s.write_fmt(format_args!("{name}=\"{value:?}\""))?;
                    }

                    s.write_str(&timestamp)?;
                }
                EventKind::LockoutChanged(v) => {
                    write_measurement_str_debug(&mut s, "safety.lockout", v, timestamp)?;
                }
                EventKind::CoolingEnableChanged(v) => {
                    write_measurement_str_debug(&mut s, "cooling.enable", v, timestamp)?;
                }
                EventKind::CoolingDemandChanged(v) => {
                    write_measurement_str_debug(&mut s, "cooling.demand", v, timestamp)?;
                }
                EventKind::Observation(ObservationEvent::TemperaturesA(v)) => {
                    write_temperature_sensors(
                        &mut s,
                        "peripheral_controller",
                        &[
                            ("onboard", v.onboard),
                            ("electronics_bay_top", v.electronics_bay_top),
                            ("laser_chamber", v.laser_chamber),
                            ("coolant_flow", v.coolant_flow),
                            ("coolant_return", v.coolant_return),
                        ],
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::AirAssistDemand(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "observation.air_assist_demand",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::ChassisIntrusion(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "observation.chassis_intrusion",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::FumeExtractionMode(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "observation.fume_extraction_mode",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::MachinePower(v)) => {
                    write_measurement_str_debug(&mut s, "observation.machine_power", v, timestamp)?;
                }
                EventKind::Observation(ObservationEvent::MachineRun(v)) => {
                    write_measurement_str_debug(&mut s, "observation.machine_run", v, timestamp)?;
                }
                EventKind::Observation(ObservationEvent::TemperaturesB(v)) => {
                    write_temperature_sensors(
                        &mut s,
                        "cooler",
                        &[
                            ("onboard", v.onboard),
                            ("internal_ambient", v.internal_ambient),
                            ("reservoir_evaporator_coil", v.reservoir_evaporator_coil),
                            ("reservoir_left_side", v.reservoir_left_side),
                            ("reservoir_right_side", v.reservoir_right_side),
                            ("coolant_pump_motor", v.coolant_pump_motor),
                        ],
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::CoolantFlow(v)) => {
                    write_measurement_numerical(
                        &mut s,
                        "observation.coolant_flow",
                        "L/min",
                        **v,
                        timestamp,
                    )?;
                }
                EventKind::Observation(ObservationEvent::CoolantReservoirLevel(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "observation.coolant_reservoir_fluid_level",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Control(ControlEvent::AirAssistPump(v)) => {
                    write_measurement_str_debug(&mut s, "control.air_assist_pump", v, timestamp)?;
                }
                EventKind::Control(ControlEvent::FumeExtractionFan(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "control.fume_extraction_fan",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Control(ControlEvent::LaserEnable(v)) => {
                    write_measurement_str_debug(&mut s, "control.laser_enable", v, timestamp)?;
                }
                EventKind::Control(ControlEvent::MachineEnable(v)) => {
                    write_measurement_str_debug(&mut s, "control.machine_enable", v, timestamp)?;
                }
                EventKind::Control(ControlEvent::StatusLamp(v)) => {
                    s.write_fmt(format_args!(
                        "control.status red=\"{}\",amber=\"{}\",green=\"{}\"{}",
                        v.red, v.amber, v.green, timestamp
                    ))?;
                }
                EventKind::Control(ControlEvent::CoolerCompressor(v)) => {
                    write_measurement_str_debug(&mut s, "control.cooler_compressor", v, timestamp)?;
                }
                EventKind::Control(ControlEvent::CoolerRadiatorFan(v)) => {
                    write_measurement_str_debug(
                        &mut s,
                        "control.cooler_radiator_fan",
                        v,
                        timestamp,
                    )?;
                }
                EventKind::Control(ControlEvent::CoolantPump(v)) => {
                    write_measurement_str_debug(&mut s, "control.coolant_pump", v, timestamp)?;
                }
            },
        }

        buffer.write_str(&s)
    }
}
