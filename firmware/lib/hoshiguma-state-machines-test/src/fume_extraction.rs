use crate::{assert_duration, assert_queue_empty};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_api::{AcBusPower, FumeExtractionFan, FumeExtractionMode, MachineRun};
use hoshiguma_state_machines::fume_extraction::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::fume_extraction::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(60), runner, async || {
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Idle)
        );

        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Running))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Run)
        );

        Timer::after_millis(500).await;

        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Idle))
            .await;
        assert_queue_empty!(communicator);

        let before = Instant::now();
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Idle)
        );
        let after = Instant::now();
        assert_duration!(
            before,
            after,
            Duration::from_secs(45),
            Duration::from_millis(50)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_mode() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::fume_extraction::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Idle)
        );

        communicator
            .send_input(InputMessage::Mode(FumeExtractionMode::OverrideRun))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Run)
        );

        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::Off))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Idle)
        );

        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Run)
        );

        communicator
            .send_input(InputMessage::Mode(FumeExtractionMode::Automatic))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::ExtractionFan(FumeExtractionFan::Idle)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
