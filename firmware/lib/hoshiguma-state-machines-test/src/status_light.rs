use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use hoshiguma_api::{
    AcBusPower, Interlock, MachineRun,
    rear_sensor_board::{LightPattern, StatusLightSettings},
};
use hoshiguma_state_machines::status_light::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::status_light::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        // Test the startup sequence and basic state changes
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;

        // Startup sequence
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::ON,
                amber: LightPattern::OFF,
                green: LightPattern::OFF,
            })
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::ON,
                green: LightPattern::OFF,
            })
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::OFF,
                green: LightPattern::ON,
            })
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::ON,
                amber: LightPattern::ON,
                green: LightPattern::ON,
            })
        );

        // After startup, the state should be updated
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::ON, // Interlock is OperationDenied by default
                amber: LightPattern::OFF,
                green: LightPattern::OFF,
            })
        );

        // Permit operation
        communicator
            .send_input(InputMessage::Interlock(Interlock::OperationPermitted))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::OFF,
                green: LightPattern::ON,
            })
        );

        // Start machine
        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Running))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::BLINK_1HZ,
                green: LightPattern::ON,
            })
        );

        // Deny continued operation
        communicator
            .send_input(InputMessage::Interlock(
                Interlock::OperationPermittedUntilIdle,
            ))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Settings(StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::BLINK_1HZ,
                green: LightPattern::OFF,
            })
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
