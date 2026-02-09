use crate::{
    devices::remote::observations::{coolant_flow_rate_rx, coolant_return_rate_rx},
    logic::interlock::update_monitor_severity,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use hoshiguma_api::Monitor;
use hoshiguma_state_machines::{
    StateMachineRun,
    coolant_rate::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::coolant_rate::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("coolant rate sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("coolant rate sm comm").await;

    let mut rate_flow_rx = coolant_flow_rate_rx();
    let mut rate_return_rx = coolant_return_rate_rx();

    loop {
        match select3(
            communicator.receive_output(),
            rate_flow_rx.changed(),
            rate_return_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::RateSeverity(severity)) => {
                update_monitor_severity(Monitor::CoolantRate, severity).await;
            }
            Either3::First(OutputMessage::SymmetrySeverity(severity)) => {
                update_monitor_severity(Monitor::CoolantRateSymmetry, severity).await;
            }
            Either3::Second(reading) => {
                communicator
                    .send_input(InputMessage::RateFlow(reading))
                    .await;
            }
            Either3::Third(reading) => {
                communicator
                    .send_input(InputMessage::RateReturn(reading))
                    .await;
            }
        }
    }
}
