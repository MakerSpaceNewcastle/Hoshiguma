use crate::devices::local::{
    ac_bus_power_detector::ac_bus_power_rx, air_assist_demand_detector::air_assist_demand_rx,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use hoshiguma_api::AirAssistPump;
use hoshiguma_state_machines::{
    StateMachineRun,
    air_assist::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::air_assist::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("air assist sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("air assist sm comm").await;

    let mut ac_bus_power_rx = ac_bus_power_rx();
    let mut air_assist_demand_rx = air_assist_demand_rx();

    let pump_tx = AIR_ASSIST_PUMP.sender();

    loop {
        match select3(
            communicator.receive_output(),
            ac_bus_power_rx.changed(),
            air_assist_demand_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::AirAssistPump(state)) => {
                pump_tx.send(state);
            }
            Either3::Second(state) => {
                communicator
                    .send_input(InputMessage::AcBusPower(state))
                    .await;
            }
            Either3::Third(state) => {
                communicator
                    .send_input(InputMessage::AirAssistDemand(state))
                    .await;
            }
        }
    }
}

crate::variable_watch!(air_assist_pump, AirAssistPump, 1);
