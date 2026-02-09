use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant};
use hoshiguma_api::{
    AirflowSensorMeasurement, AirflowSensorMeasurementInner, FumeExtractionFan, Severity,
};
use hoshiguma_common::{changed::ObservedValue, maybe_timer::MaybeTimer};

crate::state_machine!(InputMessage, OutputMessage, State, 4);

pub enum InputMessage {
    FumeExtractionFan(FumeExtractionFan),
    ExtractionAirflowReading(AirflowSensorMeasurement),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    FunctionalSeverity(Severity),
    AirflowSeverity(Severity),
}

pub struct State {
    fan_state: FumeExtractionFan,
    fan_state_change_time: Instant,

    airflow_reading: AirflowSensorMeasurementInner,
    airflow_reading_age: Instant,

    output_functional_severity: ObservedValue<Severity>,
    output_airflow_severity: ObservedValue<Severity>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fan_state: FumeExtractionFan::Idle,
            fan_state_change_time: Instant::now(),

            airflow_reading: AirflowSensorMeasurementInner::default(),
            airflow_reading_age: Instant::now(),

            output_functional_severity: ObservedValue::default(),
            output_airflow_severity: ObservedValue::default(),
        }
    }
}

/// Warning differential pressure in Pa.
const WARN: f32 = 52.0;

/// Critical differential pressure in Pa.
const CRITICAL: f32 = 45.0;

/// Amount of time it typically takes the fan to reach normal operating airflow after it is powered
/// on from stationary.
/// This can be quite conservative as very little fumes will be produced in the first few seconds
/// of a job.
const FAN_RUNUP_TIME: Duration = Duration::from_secs(4);

/// Maximum age of an airflow reading for it to be considered good.
/// Alarm will be raised if there has not been a good reading for this long.
const MAX_AGE_FOR_GOOD_READING: Duration = Duration::from_secs(20);

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            // Time to wake to send an alarm if no good reading is sent before the reading expires.
            // Only wake once to do this.
            let reading_expiry_time = self.state.airflow_reading_age + MAX_AGE_FOR_GOOD_READING;
            let reading_expiry_time = if Instant::now() < reading_expiry_time {
                Some(reading_expiry_time)
            } else {
                None
            };

            match select(
                self.input_channel.receive(),
                MaybeTimer::at(reading_expiry_time),
            )
            .await
            {
                Either::First(InputMessage::FumeExtractionFan(state)) => {
                    self.state.fan_state = state;
                    self.state.fan_state_change_time = Instant::now();
                }
                Either::First(InputMessage::ExtractionAirflowReading(Ok(state))) => {
                    self.state.airflow_reading = state;
                    self.state.airflow_reading_age = Instant::now();
                }
                Either::First(InputMessage::ExtractionAirflowReading(Err(()))) => {}
                Either::Second(_) => {
                    info!("shit");
                }
            };

            // Check age of last good reading
            let reading_age = Instant::now() - self.state.airflow_reading_age;
            let severity = if reading_age > MAX_AGE_FOR_GOOD_READING {
                Severity::Critical
            } else {
                Severity::Normal
            };
            info!(
                "reading age {}s = severity {}",
                reading_age.as_secs(),
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

            // Check reading if it is not too old
            if severity == Severity::Normal {
                let severity = match self.state.fan_state {
                    FumeExtractionFan::Idle => {
                        info!("Fan is idle, ignoring airflow reading");
                        Severity::Normal
                    }
                    FumeExtractionFan::Run => {
                        let time_fan_running = Instant::now() - self.state.fan_state_change_time;
                        let severity = if time_fan_running < FAN_RUNUP_TIME {
                            // Fan is still running up, ignore airflow reading until fan and airflow will have stabilised
                            Severity::Normal
                        } else {
                            if self.state.airflow_reading.differential_pressure > WARN {
                                Severity::Normal
                            } else if self.state.airflow_reading.differential_pressure > CRITICAL {
                                Severity::Warning
                            } else {
                                Severity::Critical
                            }
                        };
                        info!(
                            "differential pressure {} Pa, {}s after fan start = severity {}",
                            self.state.airflow_reading.differential_pressure,
                            time_fan_running.as_secs(),
                            severity
                        );
                        severity
                    }
                };

                self.state
                    .output_airflow_severity
                    .update_and_async(severity, async |v| {
                        self.output_channel
                            .send(OutputMessage::AirflowSeverity(v))
                            .await;
                    })
                    .await;
            }
        }
    }
}
