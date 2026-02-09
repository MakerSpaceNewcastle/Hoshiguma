use defmt::info;
use heapless::LinearMap;
use hoshiguma_api::{Interlock, InterlockAction, MachineRun, Monitor, Severity};
use hoshiguma_common::changed::ObservedValue;
use strum::{EnumCount, IntoEnumIterator};

crate::state_machine!(InputMessage, OutputMessage, State, 32);

pub enum InputMessage {
    Monitor(Monitor, Severity),
    MachineRun(MachineRun),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    States(MonitorStateMap),
    Interlock(Interlock),
    Action(InterlockAction),
}

pub type MonitorStateMap = LinearMap<Monitor, Severity, { Monitor::COUNT }>;

pub struct State {
    monitor_states: MonitorStateMap,
    machine_run: MachineRun,

    output_states: ObservedValue<MonitorStateMap>,
    output_interlock: ObservedValue<Interlock>,
    output_action: ObservedValue<InterlockAction>,
}

impl Default for State {
    fn default() -> Self {
        // Default to all monitors being critical until updated otherwise.
        let mut monitor_states = MonitorStateMap::new();
        for m in Monitor::iter() {
            monitor_states.insert(m, Severity::Critical).unwrap();
        }
        // Except for the interlock tripped monitor, which should default to normal.
        monitor_states
            .insert(Monitor::InterlockTripped, Severity::Normal)
            .unwrap();

        Self {
            monitor_states,
            machine_run: MachineRun::Idle,

            output_states: ObservedValue::default(),
            output_interlock: ObservedValue::default(),
            output_action: ObservedValue::default(),
        }
    }
}

impl State {
    /// Gets the overall severity across all monitors, i.e. the most severe individual monitor.
    fn overall_severity(&self) -> Severity {
        let mut severity = Severity::Normal;
        for s in self.monitor_states.values() {
            severity = core::cmp::max(severity, *s);
        }
        severity
    }
}

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match self.input_channel.receive().await {
                InputMessage::Monitor(monitor, severity) => {
                    assert_ne!(
                        monitor,
                        Monitor::InterlockTripped,
                        "Monitor::InterlockTripped should never be set from outside the interlock state machine"
                    );

                    self.state.monitor_states.insert(monitor, severity).expect(
                        "Map should never be full since we have a fixed number of monitors",
                    );

                    // Send monitor map
                    self.state
                        .output_states
                        .update_and_async(self.state.monitor_states.clone(), async |v| {
                            self.output_channel.send(OutputMessage::States(v)).await;
                        })
                        .await;

                    // Set the trip monitor if there is ever a fatal severity
                    if self.state.overall_severity() == Severity::Fatal {
                        self.state
                            .monitor_states
                            .insert(Monitor::InterlockTripped, Severity::Fatal)
                            .unwrap();
                    }
                }
                InputMessage::MachineRun(machine_run) => {
                    self.state.machine_run = machine_run;
                }
            }

            let severity = self.state.overall_severity();

            let interlock = match severity {
                Severity::Normal | Severity::Information => Interlock::OperationPermitted,
                Severity::Warning => Interlock::OperationPermittedUntilIdle,
                Severity::Critical => Interlock::OperationDenied,
                Severity::Fatal => Interlock::MachineProtected,
            };

            info!("severity {} = interlock {}", severity, interlock);

            // Send interlock output
            self.state
                .output_interlock
                .update_and_async(interlock, async |v| {
                    self.output_channel.send(OutputMessage::Interlock(v)).await;
                })
                .await;

            let action = match interlock {
                Interlock::OperationPermitted => InterlockAction::Normal,
                Interlock::OperationPermittedUntilIdle => match self.state.machine_run {
                    MachineRun::Running => InterlockAction::Normal,
                    MachineRun::Idle => InterlockAction::Disable,
                },
                Interlock::OperationDenied => InterlockAction::Disable,
                Interlock::MachineProtected => InterlockAction::Shutdown,
            };

            info!(
                "interlock {} + machine {} = action {}",
                interlock, self.state.machine_run, action
            );

            // Send action output
            self.state
                .output_action
                .update_and_async(action, async |v| {
                    self.output_channel.send(OutputMessage::Action(v)).await;
                })
                .await;
        }
    }
}
