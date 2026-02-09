use crate::devices::{
    local::ac_bus_power_detector::ac_bus_power_rx, temperature::TEMPERATURE_SENSOR_READING,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use embassy_sync::pubsub::WaitResult;
use hoshiguma_api::cooler::{CompressorState, CoolantPumpState, RadiatorFanState};
use hoshiguma_state_machines::{
    StateMachineRun,
    cooling::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::cooling::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cooling sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cooling sm comm").await;

    let mut ac_bus_power_rx = ac_bus_power_rx();
    let mut temperature_rx = TEMPERATURE_SENSOR_READING.subscriber().unwrap();

    let coolant_pump_tx = COOLANT_PUMP.sender();
    let radiator_fan_tx = RADIATOR_FAN.sender();
    let compressor_tx = COMPRESSOR.sender();

    loop {
        match select3(
            communicator.receive_output(),
            ac_bus_power_rx.changed(),
            temperature_rx.next_message(),
        )
        .await
        {
            Either3::First(OutputMessage::CoolantPump(state)) => {
                coolant_pump_tx.send(state);
            }
            Either3::First(OutputMessage::RadiatorFan(state)) => {
                radiator_fan_tx.send(state);
            }
            Either3::First(OutputMessage::Compressor(state)) => {
                compressor_tx.send(state);
            }
            Either3::Second(state) => {
                communicator
                    .send_input(InputMessage::AcBusPower(state))
                    .await;
            }
            Either3::Third(WaitResult::Message(reading)) => {
                communicator
                    .send_input(InputMessage::Temperature(reading))
                    .await;
            }
            Either3::Third(WaitResult::Lagged(n)) => {
                panic!("subscriber lagged, lost {} messages", n);
            }
        }
    }
}

crate::variable_watch!(coolant_pump, CoolantPumpState, 1);
crate::variable_watch!(radiator_fan, RadiatorFanState, 1);
crate::variable_watch!(compressor, CompressorState, 1);
