use crate::{api::access_control_state_rx, logic::interlock::interlock_action_rx};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use hoshiguma_api::DesiredMachinePower;
use hoshiguma_state_machines::{
    StateMachineRun,
    machine_power::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) =
        hoshiguma_state_machines::machine_power::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine power sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine power sm comm").await;

    let mut access_control_state_rx = access_control_state_rx();
    let mut interlock_action_rx = interlock_action_rx();

    let machine_power_tx = MACHINE_POWER.sender();

    loop {
        match select3(
            communicator.receive_output(),
            access_control_state_rx.changed(),
            interlock_action_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::Power(state)) => {
                machine_power_tx.send(state);
            }
            Either3::Second(state) => {
                communicator
                    .send_input(InputMessage::AccessControlState(state))
                    .await;
            }
            Either3::Third(state) => {
                communicator
                    .send_input(InputMessage::InterlockAction(state))
                    .await;
            }
        }
    }
}

crate::variable_watch!(machine_power, DesiredMachinePower, 3);
