use crate::{
    devices::remote::observations::extraction_airflow_rx,
    logic::{fume_extraction::fume_extraction_fan_rx, interlock::update_monitor_severity},
};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use hoshiguma_api::Monitor;
use hoshiguma_state_machines::{
    StateMachineRun,
    extraction_airflow::{
        InputChannel, InputMessage, OutputChannel, OutputMessage, StateMachineCommunicator,
        StateMachineRunner,
    },
};

static SM_INPUT: InputChannel = InputChannel::new();
static SM_OUTPUT: OutputChannel = OutputChannel::new();

pub(crate) fn init(spawner: Spawner) {
    let (runner, communicator) =
        hoshiguma_state_machines::extraction_airflow::new(&SM_INPUT, &SM_OUTPUT);

    spawner.spawn(runner_task(runner).unwrap());
    spawner.spawn(communication_task(communicator).unwrap());
}

#[embassy_executor::task]
async fn runner_task(mut runner: StateMachineRunner<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("extraction airflow sm runner").await;

    runner.run().await
}

#[embassy_executor::task]
async fn communication_task(mut communicator: StateMachineCommunicator<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("extraction airflow sm comm").await;

    let mut fume_extraction_fan_rx = fume_extraction_fan_rx();
    let mut fume_extraction_airflow_rx = extraction_airflow_rx();

    loop {
        match select3(
            communicator.receive_output(),
            fume_extraction_fan_rx.changed(),
            fume_extraction_airflow_rx.changed(),
        )
        .await
        {
            Either3::First(OutputMessage::FunctionalSeverity(severity)) => {
                update_monitor_severity(Monitor::ExtractionAirflowSensorFunctional, severity).await;
            }
            Either3::First(OutputMessage::AirflowSeverity(severity)) => {
                update_monitor_severity(Monitor::ExtractionAirflow, severity).await;
            }
            Either3::Second(state) => {
                communicator
                    .send_input(InputMessage::FumeExtractionFan(state))
                    .await;
            }
            Either3::Third(reading) => {
                communicator
                    .send_input(InputMessage::ExtractionAirflowReading(reading))
                    .await;
            }
        }
    }
}
