use crate::{assert_duration, assert_queue_empty};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use heapless::Vec;
use hoshiguma_api::{
    DesiredMachinePower, Interlock, MachineRun, Monitor, Severity,
    hmi::{AccessControlRawInput, OnscreenMessage, StatusScreenInfo},
};
use hoshiguma_state_machines::{
    hmi_status_screen::{InputMessage, OutputMessage, monitor_statuses_to_messages},
    interlock::MonitorStateMap,
};

pub(super) async fn test_states() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::hmi_status_screen::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::AccessControlRawInput(
                AccessControlRawInput::Granted,
            ))
            .await;
        let start = Instant::now();
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::Off,
                interlock: Interlock::OperationDenied,
                running: MachineRun::Idle,
                messages: Vec::new()
            })
        );
        let end = Instant::now();
        assert_duration!(
            start,
            end,
            Duration::from_millis(50),
            Duration::from_millis(5)
        );

        communicator
            .send_input(InputMessage::DesiredMachinePower(DesiredMachinePower::On))
            .await;
        let start = Instant::now();
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::On,
                interlock: Interlock::OperationDenied,
                running: MachineRun::Idle,
                messages: Vec::new()
            })
        );
        let end = Instant::now();
        assert_duration!(
            start,
            end,
            Duration::from_millis(50),
            Duration::from_millis(5)
        );

        communicator
            .send_input(InputMessage::Interlock(Interlock::OperationPermitted))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::On,
                interlock: Interlock::OperationPermitted,
                running: MachineRun::Idle,
                messages: Vec::new()
            })
        );

        communicator
            .send_input(InputMessage::DesiredMachinePower(DesiredMachinePower::On))
            .await;
        assert_queue_empty!(communicator);

        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Running))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::On,
                interlock: Interlock::OperationPermitted,
                running: MachineRun::Running,
                messages: Vec::new()
            })
        );

        assert_queue_empty!(communicator);
    })
    .await;
}

pub(super) async fn test_states_debounce() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::hmi_status_screen::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async || {
        communicator
            .send_input(InputMessage::AccessControlRawInput(
                AccessControlRawInput::Granted,
            ))
            .await;

        Timer::after_millis(45).await;
        assert_eq!(communicator.receive_channel_len(), 0);

        communicator
            .send_input(InputMessage::DesiredMachinePower(DesiredMachinePower::On))
            .await;

        Timer::after_millis(45).await;
        assert_eq!(communicator.receive_channel_len(), 0);

        communicator
            .send_input(InputMessage::Interlock(Interlock::OperationPermitted))
            .await;

        Timer::after_millis(55).await;
        assert_eq!(communicator.receive_channel_len(), 1);

        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::On,
                interlock: Interlock::OperationPermitted,
                running: MachineRun::Idle,
                messages: Vec::new()
            })
        );

        communicator
            .send_input(InputMessage::DesiredMachinePower(DesiredMachinePower::On))
            .await;

        Timer::after_millis(55).await;
        assert_eq!(communicator.receive_channel_len(), 0);

        communicator
            .send_input(InputMessage::MachineRun(MachineRun::Running))
            .await;

        Timer::after_millis(55).await;
        assert_eq!(communicator.receive_channel_len(), 1);

        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::StatusScreen(StatusScreenInfo {
                access_control: AccessControlRawInput::Granted,
                machine_power: DesiredMachinePower::On,
                interlock: Interlock::OperationPermitted,
                running: MachineRun::Running,
                messages: Vec::new()
            })
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
pub(super) async fn test_statuses_to_messages() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, _) =
        hoshiguma_state_machines::hmi_status_screen::new(&input_channel, &output_channel);

    // Yeah, the test framework is not needed here, but it provides the console output.
    crate::run_test(Duration::from_secs(10), runner, async || {
        let mut states = MonitorStateMap::new();
        states
            .insert(Monitor::ExtractionAirflow, Severity::Warning)
            .unwrap();
        states
            .insert(Monitor::TelemetryBridgeCommunication, Severity::Information)
            .unwrap();
        states.insert(Monitor::Doors, Severity::Critical).unwrap();

        let messages = monitor_statuses_to_messages(states);

        assert_eq!(
            messages,
            [
                OnscreenMessage {
                    text: "Door(s) Open".try_into().unwrap(),
                    severity: Severity::Critical,
                },
                OnscreenMessage {
                    text: "Extraction Airflow Low".try_into().unwrap(),
                    severity: Severity::Warning,
                },
                OnscreenMessage {
                    text: "Telemetry INOP".try_into().unwrap(),
                    severity: Severity::Information,
                },
            ],
        );
    })
    .await;
}
