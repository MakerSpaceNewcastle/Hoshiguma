use defmt::{Format, debug, info};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant};
use hoshiguma_api::{AcBusPower, FumeExtractionFan, FumeExtractionMode, MachineRun};
use hoshiguma_common::{changed::ObservedValue, maybe_timer::MaybeTimer};

crate::state_machine!(InputMessage, OutputMessage, State, 4);

pub enum InputMessage {
    AcBusPower(AcBusPower),
    MachineRun(MachineRun),
    Mode(FumeExtractionMode),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    ExtractionFan(FumeExtractionFan),
}

pub struct State {
    machine_power: AcBusPower,
    mode: FumeExtractionMode,
    state: RunPhase,

    output_fan: ObservedValue<FumeExtractionFan>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            machine_power: AcBusPower::Off,
            mode: FumeExtractionMode::Automatic,
            state: RunPhase::Idle,

            output_fan: ObservedValue::default(),
        }
    }
}

impl From<&State> for FumeExtractionFan {
    fn from(state: &State) -> Self {
        match state.mode {
            FumeExtractionMode::Automatic => (&state.state).into(),
            FumeExtractionMode::OverrideRun => Self::Run,
        }
    }
}

#[derive(Format, Clone)]
enum RunPhase {
    Idle,
    RunOn { until: Instant },
    Demand,
}

impl From<&RunPhase> for FumeExtractionFan {
    fn from(state: &RunPhase) -> Self {
        match state {
            RunPhase::Idle => Self::Idle,
            RunPhase::RunOn { until: _ } => Self::Run,
            RunPhase::Demand => Self::Run,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(45);

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            let run_on_timer =
                MaybeTimer::at(if let RunPhase::RunOn { until } = self.state.state {
                    Some(until)
                } else {
                    None
                });

            match select(self.input_channel.receive(), run_on_timer).await {
                Either::First(InputMessage::AcBusPower(state)) => {
                    self.state.machine_power = state;
                }
                Either::First(InputMessage::MachineRun(state)) => {
                    self.state.state = match state {
                        MachineRun::Idle => match self.state.state {
                            RunPhase::Idle => RunPhase::Idle,
                            RunPhase::RunOn { until } => RunPhase::RunOn { until },
                            RunPhase::Demand => RunPhase::RunOn {
                                until: Instant::now() + TIMEOUT,
                            },
                        },
                        MachineRun::Running => RunPhase::Demand,
                    };
                }
                Either::First(InputMessage::Mode(mode)) => {
                    self.state.mode = mode;
                }
                Either::Second(()) => {
                    debug!("Run on timer expired");
                    self.state.state = RunPhase::Idle;
                }
            }

            info!("Fume extraction fan state {}", self.state.state);

            // Turn off demand relay if the machine is not powered.
            let output = match self.state.machine_power {
                AcBusPower::Off => FumeExtractionFan::Idle,
                AcBusPower::On => (&self.state).into(),
            };

            self.state
                .output_fan
                .update_and_async(output, async |v| {
                    self.output_channel
                        .send(OutputMessage::ExtractionFan(v))
                        .await;
                })
                .await;
        }
    }
}
