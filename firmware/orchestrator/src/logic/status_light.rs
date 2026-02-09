use crate::{
    devices::local::{
        ac_bus_power_detector::ac_bus_power_rx, machine_run_detector::machine_run_rx,
    },
    logic::interlock::interlock_rx,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either4, select4};
use hoshiguma_api::rear_sensor_board::StatusLightSettings;
use hoshiguma_state_machines::{
    StateMachineRun,
    status_light::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::status_light::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("status light sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("status light sm comm").await;

    let mut ac_bus_power_rx = ac_bus_power_rx();
    let mut machine_run_rx = machine_run_rx();
    let mut interlock_rx = interlock_rx();

    let setting_tx = STATUS_LIGHT.sender();

    loop {
        match select4(
            communicator.receive_output(),
            ac_bus_power_rx.changed(),
            machine_run_rx.changed(),
            interlock_rx.changed(),
        )
        .await
        {
            Either4::First(OutputMessage::Settings(settings)) => {
                setting_tx.send(settings);
            }
            Either4::Second(state) => {
                communicator
                    .send_input(InputMessage::AcBusPower(state))
                    .await;
            }
            Either4::Third(state) => {
                communicator
                    .send_input(InputMessage::MachineRun(state))
                    .await;
            }
            Either4::Fourth(state) => {
                communicator
                    .send_input(InputMessage::Interlock(state))
                    .await;
            }
        }
    }
}

crate::variable_watch!(status_light, StatusLightSettings, 1);
