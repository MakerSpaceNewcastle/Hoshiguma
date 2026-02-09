use crate::devices::local::{
    ac_bus_power_detector::ac_bus_power_rx, machine_run_detector::machine_run_rx,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use hoshiguma_api::FumeExtractionFan;
use hoshiguma_state_machines::{
    StateMachineRun,
    fume_extraction::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) =
        hoshiguma_state_machines::fume_extraction::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("fume extraction sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("fume extraction sm comm").await;

    let mut ac_bus_power_rx = ac_bus_power_rx();
    let mut machine_run_rx = machine_run_rx();

    let fume_extraction_fan_tx = FUME_EXTRACTION_FAN.sender();

    loop {
        match select3(
            communicator.receive_output(),
            ac_bus_power_rx.changed(),
            machine_run_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::ExtractionFan(state)) => {
                fume_extraction_fan_tx.send(state);
            }
            Either3::Second(state) => {
                communicator
                    .send_input(InputMessage::AcBusPower(state))
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

crate::variable_watch!(fume_extraction_fan, FumeExtractionFan, 2);
