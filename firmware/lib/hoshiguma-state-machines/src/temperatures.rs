use defmt::{Format, debug, info, warn};
use embassy_time::{Duration, Instant};
use heapless::LinearMap;
use hoshiguma_api::{Severity, TemperatureReading, TemperatureSensor, TemperatureSensorReading};
use hoshiguma_common::changed::ObservedValue;
use strum::{EnumCount, IntoEnumIterator};

crate::state_machine!(InputMessage, OutputMessage, State, 16);

pub enum InputMessage {
    Temperature(TemperatureSensorReading),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    FunctionalSeverity(Severity),
    ElectronicsTemperatureSeverity(Severity),
    CoolantFlowTemperatureSeverity(Severity),
    CoolantReservoirTemperatureSeverity(Severity),
}

#[derive(Debug, Format, Clone, PartialEq)]
pub struct TemperatureSensorDetails {
    reading: TemperatureReading,
    last_good_reading: Instant,
}

impl Eq for TemperatureSensorDetails {}

impl Default for TemperatureSensorDetails {
    fn default() -> Self {
        Self {
            reading: Err(()),
            last_good_reading: Instant::now(),
        }
    }
}

pub type StateMap =
    LinearMap<TemperatureSensor, TemperatureSensorDetails, { TemperatureSensor::COUNT - 1 }>;

pub struct State {
    sensors: StateMap,

    output_functional_severity: ObservedValue<Severity>,
    output_electronics_temperature_severity: ObservedValue<Severity>,
    output_coolant_flow_temperature_severity: ObservedValue<Severity>,
    output_coolant_reservoir_temperature_severity: ObservedValue<Severity>,
}

impl Default for State {
    fn default() -> Self {
        let mut sensors = StateMap::new();
        for sensor in TemperatureSensor::iter() {
            if !matches!(sensor, TemperatureSensor::UnknownOnewire(_)) {
                sensors
                    .insert(sensor, TemperatureSensorDetails::default())
                    .unwrap();
            }
        }

        Self {
            sensors,

            output_functional_severity: ObservedValue::default(),
            output_electronics_temperature_severity: ObservedValue::default(),
            output_coolant_flow_temperature_severity: ObservedValue::default(),
            output_coolant_reservoir_temperature_severity: ObservedValue::default(),
        }
    }
}

const READING_AGE_WARNING_THRESHOLD: Duration = Duration::from_secs(10);
const READING_AGE_CRITICAL_THRESHOLD: Duration = Duration::from_secs(30);

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            // Receive a temperature reading and update the state for that sensor.
            let InputMessage::Temperature(sensor_reading) = self.input_channel.receive().await;

            // Ignore unknown sensors
            if matches!(sensor_reading.sensor, TemperatureSensor::UnknownOnewire(_)) {
                continue;
            }

            self.state
                .sensors
                .insert(
                    sensor_reading.sensor,
                    TemperatureSensorDetails {
                        reading: sensor_reading.reading,
                        last_good_reading: Instant::now(),
                    },
                )
                .unwrap();

            // Check for any failed sensors.
            let oldest_reading_time = self
                .state
                .sensors
                .values()
                .map(|details| details.last_good_reading)
                .min()
                .unwrap();
            let oldest_reading_age = Instant::now() - oldest_reading_time;
            let severity = if oldest_reading_age > READING_AGE_CRITICAL_THRESHOLD {
                Severity::Critical
            } else if oldest_reading_age > READING_AGE_WARNING_THRESHOLD {
                Severity::Warning
            } else {
                Severity::Normal
            };
            info!(
                "oldest reading age {}s = severity {}",
                oldest_reading_age.as_secs(),
                severity
            );
            self.state
                .output_functional_severity
                .update_and_async(severity, async |v| {
                    self.output_channel
                        .send(OutputMessage::FunctionalSeverity(v))
                        .await;
                })
                .await;

            // Check specific sensors are within acceptable ranges.
            self.state
                .output_electronics_temperature_severity
                .update_and_async(
                    check_temperatures(
                        &self.state.sensors,
                        &[
                            TemperatureSensor::OrchastratorPcb,
                            TemperatureSensor::CoolerPcb,
                        ],
                        35.0,
                        40.0,
                    ),
                    async |v| {
                        self.output_channel
                            .send(OutputMessage::ElectronicsTemperatureSeverity(v))
                            .await;
                    },
                )
                .await;

            self.state
                .output_coolant_flow_temperature_severity
                .update_and_async(
                    check_temperatures(
                        &self.state.sensors,
                        &[TemperatureSensor::CoolantFlowAtTube],
                        25.0,
                        35.0,
                    ),
                    async |v| {
                        self.output_channel
                            .send(OutputMessage::CoolantFlowTemperatureSeverity(v))
                            .await;
                    },
                )
                .await;

            self.state
                .output_coolant_reservoir_temperature_severity
                .update_and_async(
                    check_temperatures(
                        &self.state.sensors,
                        &[TemperatureSensor::CoolantReservoir],
                        20.0,
                        25.0,
                    ),
                    async |v| {
                        self.output_channel
                            .send(OutputMessage::CoolantReservoirTemperatureSeverity(v))
                            .await;
                    },
                )
                .await;
        }
    }
}

fn check_temperatures(
    readings: &StateMap,
    sensors: &[TemperatureSensor],
    warn: f32,
    critical: f32,
) -> Severity {
    sensors
        .iter()
        .map(|sensor| {
            temperature_to_severity(warn, critical, readings.get(sensor).unwrap().reading)
        })
        .fold(Severity::Normal, |acc, severity| {
            if let Ok(severity) = severity {
                acc.max(severity)
            } else {
                Severity::Critical
            }
        })
}

fn temperature_to_severity(
    warn: f32,
    critical: f32,
    temperature: TemperatureReading,
) -> Result<Severity, ()> {
    if let Ok(temperature) = temperature {
        Ok(if temperature >= critical {
            warn!(
                "Temperature {} is above critical threshold of {}",
                temperature, critical
            );
            Severity::Critical
        } else if temperature >= warn {
            warn!(
                "Temperature {} is above warning threshold of {}",
                temperature, warn
            );
            Severity::Warning
        } else {
            debug!("Temperature {} is normal", temperature);
            Severity::Normal
        })
    } else {
        warn!("Asked to check temperature of a sensor that failed to be read");
        Err(())
    }
}
