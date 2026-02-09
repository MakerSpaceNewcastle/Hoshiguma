use crate::{assert_duration, assert_queue_empty};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_api::{AcBusPower, AirAssistDemand, AirAssistPump};
use hoshiguma_state_machines::air_assist::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::air_assist::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirAssistPump(AirAssistPump::Idle)
        );

        communicator
            .send_input(InputMessage::AirAssistDemand(AirAssistDemand::Demand))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirAssistPump(AirAssistPump::Run)
        );

        Timer::after_millis(500).await;

        communicator
            .send_input(InputMessage::AirAssistDemand(AirAssistDemand::Idle))
            .await;
        assert_queue_empty!(communicator);

        let before = Instant::now();
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirAssistPump(AirAssistPump::Idle)
        );
        let after = Instant::now();
        assert_duration!(
            before,
            after,
            Duration::from_secs(1),
            Duration::from_millis(50)
        );

        communicator
            .send_input(InputMessage::AirAssistDemand(AirAssistDemand::Demand))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirAssistPump(AirAssistPump::Run)
        );

        communicator
            .send_input(InputMessage::AirAssistDemand(AirAssistDemand::Idle))
            .await;
        assert_queue_empty!(communicator);

        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::Off))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::AirAssistPump(AirAssistPump::Idle)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
