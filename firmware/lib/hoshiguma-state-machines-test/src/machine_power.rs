use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use hoshiguma_api::{DesiredMachinePower, InterlockAction, hmi::AccessControlState};
use hoshiguma_state_machines::machine_power::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::machine_power::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::InterlockAction(InterlockAction::Disable))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Power(DesiredMachinePower::Off)
        );

        communicator
            .send_input(InputMessage::AccessControlState(AccessControlState::Denied))
            .await;
        assert_queue_empty!(communicator);

        communicator
            .send_input(InputMessage::AccessControlState(
                AccessControlState::Granted,
            ))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Power(DesiredMachinePower::On)
        );

        communicator
            .send_input(InputMessage::InterlockAction(InterlockAction::Disable))
            .await;
        assert_queue_empty!(communicator);

        communicator
            .send_input(InputMessage::InterlockAction(InterlockAction::Shutdown))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Power(DesiredMachinePower::Off)
        );

        communicator
            .send_input(InputMessage::InterlockAction(InterlockAction::Normal))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Power(DesiredMachinePower::On)
        );

        communicator
            .send_input(InputMessage::AccessControlState(AccessControlState::Denied))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Power(DesiredMachinePower::Off)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
