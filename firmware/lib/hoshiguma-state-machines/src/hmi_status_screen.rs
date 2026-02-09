use crate::interlock::MonitorStateMap;
use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant};
use heapless::Vec;
use hoshiguma_api::{
    DesiredMachinePower, Interlock, MachineRun, Monitor, Severity,
    hmi::{AccessControlRawInput, OnscreenMessage, StatusScreenInfo},
};
use hoshiguma_common::{changed::ObservedValue, maybe_timer::MaybeTimer};
use strum::EnumCount;

crate::state_machine!(InputMessage, OutputMessage, State, 8);

pub enum InputMessage {
    AccessControlRawInput(AccessControlRawInput),
    DesiredMachinePower(DesiredMachinePower),
    Interlock(Interlock),
    MachineRun(MachineRun),
    MonitorStates(MonitorStateMap),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    StatusScreen(StatusScreenInfo),
}

pub struct State {
    current: StatusScreenInfo,
    next_emit_time: Option<Instant>,
    last_emitted: ObservedValue<StatusScreenInfo>,
}

impl Default for State {
    fn default() -> Self {
        let default_status = StatusScreenInfo {
            access_control: AccessControlRawInput::Idle,
            machine_power: DesiredMachinePower::Off,
            interlock: Interlock::OperationDenied,
            running: MachineRun::Idle,
            messages: Vec::new(),
        };

        Self {
            current: default_status,
            next_emit_time: None,
            last_emitted: ObservedValue::default(),
        }
    }
}

const DEBOUNCE_DURATION: Duration = Duration::from_millis(50);

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match select(
                self.input_channel.receive(),
                MaybeTimer::at(self.state.next_emit_time),
            )
            .await
            {
                Either::First(input) => {
                    match input {
                        InputMessage::AccessControlRawInput(state) => {
                            self.state.current.access_control = state;
                        }
                        InputMessage::DesiredMachinePower(state) => {
                            self.state.current.machine_power = state;
                        }
                        InputMessage::Interlock(state) => {
                            self.state.current.interlock = state;
                        }
                        InputMessage::MachineRun(state) => {
                            self.state.current.running = state;
                        }
                        InputMessage::MonitorStates(states) => {
                            self.state.current.messages = monitor_statuses_to_messages(states);
                        }
                    }

                    self.state.next_emit_time = Some(Instant::now() + DEBOUNCE_DURATION);
                    info!(
                        "Received new input, scheduling status screen emit at {}",
                        self.state.next_emit_time
                    );
                }
                Either::Second(()) => {
                    info!(
                        "Status screen debounce timer expired, emitting new status screen ({})",
                        Instant::now()
                    );
                    self.state.next_emit_time = None;
                    self.state
                        .last_emitted
                        .update_and_async(self.state.current.clone(), async |v| {
                            self.output_channel
                                .send(OutputMessage::StatusScreen(v))
                                .await;
                        })
                        .await;
                }
            }
        }
    }
}

pub fn monitor_statuses_to_messages(monitor_states: MonitorStateMap) -> Vec<OnscreenMessage, 8> {
    // First sort the states by severity, most severe first.
    let mut monitor_states: Vec<_, { Monitor::COUNT }> = monitor_states.into_iter().collect();
    monitor_states.sort_unstable_by_key(|i| core::cmp::Reverse(i.1));

    // Then convert to messages, skipping any that are normal, with a max of 8 messages.
    monitor_states
        .into_iter()
        .take(8)
        .filter(|(_, severity)| *severity != Severity::Normal)
        .map(|(monitor, severity)| OnscreenMessage {
            text: monitor_to_text(monitor).try_into().unwrap(),
            severity,
        })
        .collect()
}

fn monitor_to_text(monitor: Monitor) -> &'static str {
    match monitor {
        Monitor::InterlockTripped => "Interlock Tripped",
        Monitor::AcBusPower => "AC Bus Off",
        Monitor::Doors => "Door(s) Open",
        Monitor::CoolerCommunication => "Cooler INOP",
        Monitor::RearSensorBoardCommunication => "Rear Board INOP",
        Monitor::HmiCommunication => "HMI INOP",
        Monitor::TelemetryBridgeCommunication => "Telemetry INOP",
        Monitor::CoolantRateSymmetry => "Coolant Rate Asymmetry",
        Monitor::CoolantRate => "Coolant Rate Low",
        Monitor::TemperatureSensorsFunctional => "Temperature Sensor Fault",
        Monitor::ElectronicsTemperature => "Electronics Temp.",
        Monitor::CoolantFlowTemperature => "Coolant Flow Temp.",
        Monitor::CoolantReservoirTemperature => "Coolant Reservoir Temp.",
        Monitor::ExtractionAirflow => "Extraction Airflow Low",
        Monitor::ExtractionAirflowSensorFunctional => "Airflow Sensor Fault",
    }
}
