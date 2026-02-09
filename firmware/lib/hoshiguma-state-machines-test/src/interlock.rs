use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use hoshiguma_api::{Interlock, InterlockAction, MachineRun, Monitor, Severity};
use hoshiguma_state_machines::interlock::{InputMessage, OutputMessage};
use strum::{EnumCount, IntoEnumIterator};

pub(super) async fn test_init_denied() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::interlock::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Idle))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Interlock(Interlock::OperationDenied)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Action(InterlockAction::Disable)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_become_happy_then_get_sad() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::interlock::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        for monitor in Monitor::iter() {
            if monitor == Monitor::InterlockTripped {
                continue;
            }

            communicator
                .send_input(InputMessage::Monitor(monitor, Severity::Normal))
                .await;

            assert!(matches!(
                communicator.receive_output().await,
                OutputMessage::States(_)
            ));

            const FIRST_MONITOR: Monitor = Monitor::AcBusPower;
            const LAST_MONITOR: Monitor = Monitor::ExtractionAirflowSensorFunctional;

            if monitor == FIRST_MONITOR {
                assert_eq!(
                    communicator.receive_output().await,
                    OutputMessage::Interlock(Interlock::OperationDenied)
                );
                assert_eq!(
                    communicator.receive_output().await,
                    OutputMessage::Action(InterlockAction::Disable)
                );
            } else if monitor == LAST_MONITOR {
                assert_eq!(
                    communicator.receive_output().await,
                    OutputMessage::Interlock(Interlock::OperationPermitted)
                );
                assert_eq!(
                    communicator.receive_output().await,
                    OutputMessage::Action(InterlockAction::Normal)
                );
            }
        }

        communicator
            .send_input(InputMessage::Monitor(Monitor::Doors, Severity::Critical))
            .await;
        assert!(matches!(
            communicator.receive_output().await,
            OutputMessage::States(_)
        ));
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Interlock(Interlock::OperationDenied)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Action(InterlockAction::Disable)
        );

        communicator
            .send_input(InputMessage::Monitor(Monitor::Doors, Severity::Normal))
            .await;
        assert!(matches!(
            communicator.receive_output().await,
            OutputMessage::States(_)
        ));
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Interlock(Interlock::OperationPermitted)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Action(InterlockAction::Normal)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_lockout() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::interlock::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        for monitor in Monitor::iter() {
            if monitor == Monitor::InterlockTripped {
                continue;
            }

            communicator
                .send_input(InputMessage::Monitor(monitor, Severity::Normal))
                .await;
        }

        // Each monitor update should generate a state update message, but we don't send the trip monitor.
        // The first and last monitor updates will generate a full set of messages (2 extra each).
        Timer::after_millis(10).await;
        assert_eq!(
            communicator.receive_channel_len(),
            2 + 2 + Monitor::COUNT - 1
        );

        // Consume all those mesaages that we do not care about for this test.
        while communicator.receive_channel_len() > 0 {
            let _ = communicator.receive_output().await;
        }

        // Simulate a right proper fuck up that should shut down the machine
        communicator
            .send_input(InputMessage::Monitor(
                Monitor::CoolantRateSymmetry,
                Severity::Fatal,
            ))
            .await;
        assert!(matches!(
            communicator.receive_output().await,
            OutputMessage::States(_)
        ));
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Interlock(Interlock::MachineProtected)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Action(InterlockAction::Shutdown)
        );

        // Fault condition goes away and yet the machine should still not work
        communicator
            .send_input(InputMessage::Monitor(
                Monitor::CoolantRateSymmetry,
                Severity::Normal,
            ))
            .await;
        assert!(matches!(
            communicator.receive_output().await,
            OutputMessage::States(_)
        ));

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_allow_until_idle() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::interlock::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        for monitor in Monitor::iter() {
            if monitor == Monitor::InterlockTripped {
                continue;
            }

            communicator
                .send_input(InputMessage::Monitor(monitor, Severity::Normal))
                .await;
        }

        // Each monitor update should generate a state update message, but we don't send the trip monitor.
        // The first and last monitor updates will generate a full set of messages (2 extra each).
        Timer::after_millis(10).await;
        assert_eq!(
            communicator.receive_channel_len(),
            2 + 2 + Monitor::COUNT - 1
        );

        // Consume all those mesaages that we do not care about for this test.
        while communicator.receive_channel_len() > 0 {
            let _ = communicator.receive_output().await;
        }

        // Simulate the machine running a job
        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Running))
            .await;
        assert_queue_empty!(communicator);

        // Simulate a minor fuck up that should allow the machine to finish the current job
        communicator
            .send_input(InputMessage::Monitor(
                Monitor::CoolantFlowTemperature,
                Severity::Warning,
            ))
            .await;
        assert!(matches!(
            communicator.receive_output().await,
            OutputMessage::States(_)
        ));
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Interlock(Interlock::OperationPermittedUntilIdle)
        );

        // The machine finishes, but the fault condition did not go away.
        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Idle))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Action(InterlockAction::Disable)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
