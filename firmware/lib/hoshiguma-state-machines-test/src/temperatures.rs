use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use hoshiguma_api::{Severity, TemperatureSensor, TemperatureSensorReading};
use hoshiguma_state_machines::temperatures::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::temperatures::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        // The very first temperature reading triggers all four output messages because
        // the ObservedValues are uninitialised. Unupdated sensors start with
        // reading = Err(()), which drives the three temperature-severity outputs to
        // Critical. The functional severity is Normal because the "oldest reading age"
        // (measured from Instant::MIN = 0 for sensors that have never reported) is
        // just the elapsed time since device boot – well under 10 s during tests.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::OrchastratorPcb,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::FunctionalSeverity(Severity::Normal)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Critical)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Critical)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantReservoirTemperatureSeverity(Severity::Critical)
        );

        // The second electronics sensor reports normally; both sensors are healthy so
        // the electronics severity resolves to Normal.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolerPcb,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);

        // The coolant-flow sensor reports normally; its severity resolves to Normal.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);

        // The coolant-reservoir sensor reports normally; its severity resolves to Normal.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(15.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantReservoirTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);

        // Sensors that are not used in any severity check produce no output when they
        // first report.
        for sensor in [TemperatureSensor::CoolantReturnAtTube] {
            communicator
                .send_input(InputMessage::Temperature(TemperatureSensorReading {
                    sensor,
                    reading: Ok(20.0),
                }))
                .await;
        }
        assert_queue_empty!(communicator);
    })
    .await;
}

/// Sends baseline readings for every tracked sensor and drains all resulting output
/// messages so subsequent assertions start with all severities at Normal and every
/// ObservedValue fully initialised.
async fn baseline_all_sensors(
    communicator: &mut hoshiguma_state_machines::temperatures::StateMachineCommunicator<'_>,
) {
    for (sensor, temp) in [
        (TemperatureSensor::OrchastratorPcb, 20.0_f32),
        (TemperatureSensor::CoolerPcb, 20.0),
        (TemperatureSensor::CoolantFlowAtTube, 20.0),
        (TemperatureSensor::CoolantReservoir, 15.0),
        (TemperatureSensor::CoolantReturnAtTube, 20.0),
    ] {
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor,
                reading: Ok(temp),
            }))
            .await;
    }
    Timer::after_millis(10).await;
    while communicator.receive_channel_len() > 0 {
        communicator.receive_output().await;
    }
}

pub(super) async fn test_electronics_temperature() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::temperatures::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        baseline_all_sensors(&mut communicator).await;

        // Temperature rises to warning level on the orchestrator PCB (warn threshold: 35 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::OrchastratorPcb,
                reading: Ok(36.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Warning)
        );
        assert_queue_empty!(communicator);

        // Temperature rises to critical level (critical threshold: 40 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::OrchastratorPcb,
                reading: Ok(41.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Critical)
        );
        assert_queue_empty!(communicator);

        // Temperature returns to normal (< 35 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::OrchastratorPcb,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);

        // The cooler PCB sensor independently drives the severity; the overall severity
        // is the maximum across both electronics sensors.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolerPcb,
                reading: Ok(38.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ElectronicsTemperatureSeverity(Severity::Warning)
        );
        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_coolant_flow_temperature() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::temperatures::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        baseline_all_sensors(&mut communicator).await;

        // Temperature rises to warning level (warn threshold: 25 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(26.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Warning)
        );
        assert_queue_empty!(communicator);

        // Temperature rises to critical level (critical threshold: 35 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(36.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Critical)
        );
        assert_queue_empty!(communicator);

        // Temperature returns to normal (< 25 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_coolant_reservoir_temperature() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::temperatures::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        baseline_all_sensors(&mut communicator).await;

        // Temperature rises to warning level (warn threshold: 20 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(21.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantReservoirTemperatureSeverity(Severity::Warning)
        );
        assert_queue_empty!(communicator);

        // Temperature rises to critical level (critical threshold: 25 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(26.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantReservoirTemperatureSeverity(Severity::Critical)
        );
        assert_queue_empty!(communicator);

        // Temperature returns to normal (< 20 °C).
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(15.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantReservoirTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_failed_sensor() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::temperatures::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        baseline_all_sensors(&mut communicator).await;

        // A sensor that fails to read causes its group's severity to go Critical.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Err(()),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Critical)
        );
        assert_queue_empty!(communicator);

        // When the sensor recovers and reports a valid reading, severity returns to Normal.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantFlowTemperatureSeverity(Severity::Normal)
        );
        assert_queue_empty!(communicator);
    })
    .await;
}
