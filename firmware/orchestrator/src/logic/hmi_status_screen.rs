use crate::{
    api::access_control_raw_input_rx,
    devices::local::machine_run_detector::machine_run_rx,
    logic::{
        interlock::{interlock_rx, monitor_states_rx},
        machine_power::machine_power_rx,
    },
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either6, select6};
use hoshiguma_api::hmi::StatusScreenInfo;
use hoshiguma_state_machines::{
    StateMachineRun,
    hmi_status_screen::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) =
        hoshiguma_state_machines::hmi_status_screen::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("hmi status screen sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("hmi status screen sm comm").await;

    let mut access_control_raw_input_rx = access_control_raw_input_rx();
    let mut desired_machine_power_rx = machine_power_rx();
    let mut interlock_rx = interlock_rx();
    let mut machine_run_rx = machine_run_rx();
    let mut monitor_states_rx = monitor_states_rx();

    let status_screen_tx = HMI_STATUS_SCREEN_INFO.sender();

    loop {
        match select6(
            communicator.receive_output(),
            access_control_raw_input_rx.changed(),
            desired_machine_power_rx.changed(),
            interlock_rx.changed(),
            machine_run_rx.changed(),
            monitor_states_rx.changed(),
        )
        .await
        {
            Either6::First(OutputMessage::StatusScreen(info)) => {
                status_screen_tx.send(info);
            }
            Either6::Second(state) => {
                communicator
                    .send_input(InputMessage::AccessControlRawInput(state))
                    .await;
            }
            Either6::Third(state) => {
                communicator
                    .send_input(InputMessage::DesiredMachinePower(state))
                    .await;
            }
            Either6::Fourth(state) => {
                communicator
                    .send_input(InputMessage::Interlock(state))
                    .await;
            }
            Either6::Fifth(state) => {
                communicator
                    .send_input(InputMessage::MachineRun(state))
                    .await;
            }
            Either6::Sixth(states) => {
                communicator
                    .send_input(InputMessage::MonitorStates(states))
                    .await;
            }
        }
    }
}

crate::variable_watch!(hmi_status_screen_info, StatusScreenInfo, 1);
