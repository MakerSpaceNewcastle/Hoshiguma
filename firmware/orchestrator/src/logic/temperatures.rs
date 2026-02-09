use crate::{
    devices::temperature::TEMPERATURE_SENSOR_READING, logic::interlock::update_monitor_severity,
    telemetry::queue_telemetry_data_point,
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::pubsub::WaitResult;
use hoshiguma_api::Monitor;
use hoshiguma_common::telemetry::format_influx_line;
use hoshiguma_state_machines::{
    StateMachineRun,
    temperatures::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) = hoshiguma_state_machines::temperatures::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("temperatures sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("temperatures sm comm").await;

    let mut temperature_rx = TEMPERATURE_SENSOR_READING.subscriber().unwrap();

    loop {
        match select(communicator.receive_output(), temperature_rx.next_message()).await {
            Either::First(OutputMessage::FunctionalSeverity(severity)) => {
                update_monitor_severity(Monitor::TemperatureSensorsFunctional, severity).await;
            }
            Either::First(OutputMessage::ElectronicsTemperatureSeverity(severity)) => {
                update_monitor_severity(Monitor::ElectronicsTemperature, severity).await;
            }
            Either::First(OutputMessage::CoolantFlowTemperatureSeverity(severity)) => {
                update_monitor_severity(Monitor::CoolantFlowTemperature, severity).await;
            }
            Either::First(OutputMessage::CoolantReservoirTemperatureSeverity(severity)) => {
                update_monitor_severity(Monitor::CoolantReservoirTemperature, severity).await;
            }
            Either::Second(WaitResult::Message(reading)) => {
                communicator
                    .send_input(InputMessage::Temperature(reading))
                    .await;

                // Submit sensor telemetry
                let sensor = reading.sensor;
                if let Ok(reading) = reading.reading {
                    queue_telemetry_data_point(format_influx_line(
                        format_args!("temperature,sensor={sensor} value={reading}"),
                        crate::wall_time::now(),
                    ));
                }
            }
            Either::Second(WaitResult::Lagged(n)) => {
                panic!("subscriber lagged, lost {} messages", n);
            }
        }
    }
}
