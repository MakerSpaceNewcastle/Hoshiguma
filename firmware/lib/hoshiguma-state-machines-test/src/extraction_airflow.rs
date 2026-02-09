use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use hoshiguma_api::{AirflowSensorMeasurementInner, FumeExtractionFan, Severity};
use hoshiguma_state_machines::extraction_airflow::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::extraction_airflow::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(40), runner, async || {
        // Send a close to zero reading
        communicator
            .send_input(InputMessage::ExtractionAirflowReading(Ok(
                AirflowSensorMeasurementInner {
                    differential_pressure: 2.0,
                    temperature: 0.0,
                },
            )))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::FunctionalSeverity(Severity::Normal)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirflowSeverity(Severity::Normal)
        );

        // Simulate fan demand
        communicator
            .send_input(InputMessage::FumeExtractionFan(FumeExtractionFan::Run))
            .await;
        assert_queue_empty!(communicator);

        // Wait for fan to run up
        embassy_time::Timer::after(Duration::from_secs(5)).await;

        // Send a good reading
        communicator
            .send_input(InputMessage::ExtractionAirflowReading(Ok(
                AirflowSensorMeasurementInner {
                    differential_pressure: 90.0,
                    temperature: 0.0,
                },
            )))
            .await;
        assert_queue_empty!(communicator);

        // Send a warning reading
        communicator
            .send_input(InputMessage::ExtractionAirflowReading(Ok(
                AirflowSensorMeasurementInner {
                    differential_pressure: 80.0,
                    temperature: 0.0,
                },
            )))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirflowSeverity(Severity::Warning)
        );
        assert_queue_empty!(communicator);

        // Send a critical reading
        communicator
            .send_input(InputMessage::ExtractionAirflowReading(Ok(
                AirflowSensorMeasurementInner {
                    differential_pressure: 60.0,
                    temperature: 0.0,
                },
            )))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirflowSeverity(Severity::Critical)
        );
        assert_queue_empty!(communicator);

        // Let reading go stale
        embassy_time::Timer::after(Duration::from_secs(21)).await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::FunctionalSeverity(Severity::Critical)
        );

        // Set fan to idle
        communicator
            .send_input(InputMessage::FumeExtractionFan(FumeExtractionFan::Idle))
            .await;
        assert_queue_empty!(communicator);

        // Send a close to zero reading
        communicator
            .send_input(InputMessage::ExtractionAirflowReading(Ok(
                AirflowSensorMeasurementInner {
                    differential_pressure: 2.0,
                    temperature: 0.0,
                },
            )))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::FunctionalSeverity(Severity::Normal)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirflowSeverity(Severity::Normal)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
