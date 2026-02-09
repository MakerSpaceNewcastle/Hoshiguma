use crate::{
    devices::local::machine_run_detector::machine_run_rx, telemetry::queue_telemetry_data_point,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use hoshiguma_api::{Interlock, InterlockAction, Monitor, Severity};
use hoshiguma_common::telemetry::format_influx_line;
use hoshiguma_state_machines::{
    StateMachineRun,
    interlock::{
        InputChannel, InputMessage, MonitorStateMap, OutputChannel, OutputMessage,
        StateMachineCommunicator, StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::interlock::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("interlock sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("interlock sm comm").await;

    let monitor_severity_rx = MONITOR_SEVERITY_CH.receiver();
    let mut machine_run_rx = machine_run_rx();

    let monitor_states_tx = MONITOR_STATES.sender();
    let interlock_tx = INTERLOCK.sender();
    let interlock_action_tx = INTERLOCK_ACTION.sender();

    loop {
        match select3(
            communicator.receive_output(),
            monitor_severity_rx.receive(),
            machine_run_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::States(states)) => {
                for (monitor, severity) in &states {
                    queue_telemetry_data_point(format_influx_line(
                        format_args!("monitor,monitor={monitor} severity=\"{severity}\""),
                        crate::wall_time::now(),
                    ));
                }

                monitor_states_tx.send(states);
            }
            Either3::First(OutputMessage::Interlock(state)) => {
                interlock_tx.send(state);

                queue_telemetry_data_point(format_influx_line(
                    format_args!("interlock value=\"{state}\""),
                    crate::wall_time::now(),
                ));
            }
            Either3::First(OutputMessage::Action(action)) => {
                interlock_action_tx.send(action);

                queue_telemetry_data_point(format_influx_line(
                    format_args!("interlock_action value=\"{action}\""),
                    crate::wall_time::now(),
                ));
            }
            Either3::Second((monitor, severity)) => {
                communicator
                    .send_input(InputMessage::Monitor(monitor, severity))
                    .await;
            }
            Either3::Third(state) => {
                communicator
                    .send_input(InputMessage::MachineRun(state))
                    .await;
            }
        }
    }
}

static MONITOR_SEVERITY_CH: Channel<CriticalSectionRawMutex, (Monitor, Severity), 32> =
    Channel::new();

pub(crate) async fn update_monitor_severity(monitor: Monitor, severity: Severity) {
    MONITOR_SEVERITY_CH.send((monitor, severity)).await;
}

crate::variable_watch!(monitor_states, MonitorStateMap, 1);
crate::variable_watch!(interlock, Interlock, 2);
crate::variable_watch!(interlock_action, InterlockAction, 3);
