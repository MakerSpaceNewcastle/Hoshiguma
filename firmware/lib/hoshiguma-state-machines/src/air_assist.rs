use defmt::{Format, debug, info};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant};
use hoshiguma_api::{AcBusPower, AirAssistDemand, AirAssistPump};
use hoshiguma_common::{changed::ObservedValue, maybe_timer::MaybeTimer};

crate::state_machine!(InputMessage, OutputMessage, State, 4);

pub enum InputMessage {
    AcBusPower(AcBusPower),
    AirAssistDemand(AirAssistDemand),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    AirAssistPump(AirAssistPump),
}

pub struct State {
    machine_power: AcBusPower,
    state: RunPhase,

    output_pump: ObservedValue<AirAssistPump>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            machine_power: AcBusPower::Off,
            state: RunPhase::Idle,

            output_pump: ObservedValue::default(),
        }
    }
}

#[derive(Format, Clone)]
enum RunPhase {
    Idle,
    RunOn { until: Instant },
    Demand,
}

impl From<&RunPhase> for AirAssistPump {
    fn from(state: &RunPhase) -> Self {
        match state {
            RunPhase::Idle => Self::Idle,
            RunPhase::RunOn { until: _ } => Self::Run,
            RunPhase::Demand => Self::Run,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(1);

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
                Either::First(InputMessage::AcBusPower(power)) => {
                    self.state.machine_power = power;
                }
                Either::First(InputMessage::AirAssistDemand(demand)) => {
                    self.state.state = match demand {
                        AirAssistDemand::Idle => match self.state.state {
                            RunPhase::Idle => RunPhase::Idle,
                            RunPhase::RunOn { until } => RunPhase::RunOn { until },
                            RunPhase::Demand => RunPhase::RunOn {
                                until: Instant::now() + TIMEOUT,
                            },
                        },
                        AirAssistDemand::Demand => RunPhase::Demand,
                    };
                }
                Either::Second(()) => {
                    debug!("Run on timer expired");
                    self.state.state = RunPhase::Idle;
                }
            }

            info!("Air assist state {}", self.state.state);

            // Turn off demand relay if the machine is not powered.
            let output = match self.state.machine_power {
                AcBusPower::Off => AirAssistPump::Idle,
                AcBusPower::On => (&self.state.state).into(),
            };

            self.state
                .output_pump
                .update_and_async(output, async |v| {
                    self.output_channel
                        .send(OutputMessage::AirAssistPump(v))
                        .await;
                })
                .await;
        }
    }
}
