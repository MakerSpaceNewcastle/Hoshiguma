use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use hoshiguma_api::{Severity, cooler::CoolantRate};
use hoshiguma_state_machines::coolant_rate::{InputMessage, OutputMessage};

pub(super) async fn test_rate() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::coolant_rate::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async move || {
        communicator
            .send_input(InputMessage::RateFlow(CoolantRate::new(2.5)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Warning)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Critical)
        );

        communicator
            .send_input(InputMessage::RateReturn(CoolantRate::new(2.48)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Normal)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_rate_symmetry_pump_start() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::coolant_rate::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async move || {
        communicator
            .send_input(InputMessage::CoolantPumpState(
                hoshiguma_api::cooler::CoolantPumpState::Run,
            ))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Critical)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Critical)
        );

        Timer::after_millis(5100).await;

        communicator
            .send_input(InputMessage::RateFlow(CoolantRate::new(5.2)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Normal)
        );

        communicator
            .send_input(InputMessage::RateReturn(CoolantRate::new(5.2)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Normal)
        );

        communicator
            .send_input(InputMessage::RateReturn(CoolantRate::new(3.5)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Fatal)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_rate_symmetry_pump_stop() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::coolant_rate::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async move || {
        // Start the pump - expect critical severities as no rates are present yet
        communicator
            .send_input(InputMessage::CoolantPumpState(
                hoshiguma_api::cooler::CoolantPumpState::Run,
            ))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Critical)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Critical)
        );

        Timer::after_millis(5100).await;

        // Set equal flow and return rates
        communicator
            .send_input(InputMessage::RateFlow(CoolantRate::new(5.2)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Normal)
        );

        communicator
            .send_input(InputMessage::RateReturn(CoolantRate::new(5.2)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::SymmetrySeverity(Severity::Normal)
        );

        // Stop the pump
        communicator
            .send_input(InputMessage::CoolantPumpState(
                hoshiguma_api::cooler::CoolantPumpState::Idle,
            ))
            .await;

        // Return rate drops to zero while flow remains high - an imbalance (diff=5.2)
        // that would be Fatal severity if the pump were still running
        communicator
            .send_input(InputMessage::RateReturn(CoolantRate::new(0.0)))
            .await;
        assert_queue_empty!(communicator);

        // Flow also drops to zero
        communicator
            .send_input(InputMessage::RateFlow(CoolantRate::new(0.0)))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RateSeverity(Severity::Critical)
        );

        // Symmetry severity remains Normal throughout because the pump is idle
        assert_queue_empty!(communicator);
    })
    .await;
}
