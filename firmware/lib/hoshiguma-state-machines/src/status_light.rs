use defmt::debug;
use embassy_time::{Duration, Ticker};
use hoshiguma_api::{
    AcBusPower, Interlock, MachineRun,
    rear_sensor_board::{LightPattern, StatusLightSettings},
};
use hoshiguma_common::changed::ObservedValue;

crate::state_machine!(InputMessage, OutputMessage, State, 8);

pub enum InputMessage {
    AcBusPower(AcBusPower),
    MachineRun(MachineRun),
    Interlock(Interlock),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    Settings(StatusLightSettings),
}

pub struct State {
    power: AcBusPower,
    run: MachineRun,
    interlock: Interlock,

    output_settings: ObservedValue<StatusLightSettings>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            power: AcBusPower::Off,
            run: MachineRun::Idle,
            interlock: Interlock::OperationDenied,

            output_settings: ObservedValue::default(),
        }
    }
}

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match self.input_channel.receive().await {
                InputMessage::AcBusPower(state) => {
                    if state == AcBusPower::On {
                        startup_sequence(self.output_channel).await;
                    }

                    self.state.power = state;
                }
                InputMessage::MachineRun(state) => {
                    self.state.run = state;
                }
                InputMessage::Interlock(state) => {
                    self.state.interlock = state;
                }
            }

            let settings = match self.state.power {
                AcBusPower::On => StatusLightSettings {
                    // Red lamp is lit when operation of the machine is denied
                    red: if self.state.run == MachineRun::Running {
                        match self.state.interlock {
                            Interlock::OperationDenied => LightPattern::ON,
                            _ => LightPattern::OFF,
                        }
                    } else {
                        match self.state.interlock {
                            Interlock::OperationDenied | Interlock::OperationPermittedUntilIdle => {
                                LightPattern::ON
                            }
                            _ => LightPattern::OFF,
                        }
                    },
                    // Amber light flashes when machine is running
                    amber: match self.state.run {
                        MachineRun::Idle => LightPattern::OFF,
                        MachineRun::Running => LightPattern::BLINK_1HZ,
                    },
                    // Green lamp is lit when operation of the machine is permitted and will continue to be permitted
                    green: match self.state.interlock {
                        Interlock::OperationPermitted => LightPattern::ON,
                        _ => LightPattern::OFF,
                    },
                },
                // All lights off when AC bus is off
                AcBusPower::Off => StatusLightSettings {
                    red: LightPattern::OFF,
                    amber: LightPattern::OFF,
                    green: LightPattern::OFF,
                },
            };
            debug!("{}", settings);

            self.state
                .output_settings
                .update_and_async(settings, async |v| {
                    self.output_channel.send(OutputMessage::Settings(v)).await;
                })
                .await;
        }
    }
}

async fn startup_sequence<'a>(output_channel: OutputChannelSender<'a>) {
    let mut tick = Ticker::every(Duration::from_millis(200));

    output_channel
        .send(OutputMessage::Settings(StatusLightSettings {
            red: LightPattern::ON,
            amber: LightPattern::OFF,
            green: LightPattern::OFF,
        }))
        .await;

    tick.next().await;

    output_channel
        .send(OutputMessage::Settings(StatusLightSettings {
            red: LightPattern::OFF,
            amber: LightPattern::ON,
            green: LightPattern::OFF,
        }))
        .await;

    tick.next().await;

    output_channel
        .send(OutputMessage::Settings(StatusLightSettings {
            red: LightPattern::OFF,
            amber: LightPattern::OFF,
            green: LightPattern::ON,
        }))
        .await;

    tick.next().await;

    output_channel
        .send(OutputMessage::Settings(StatusLightSettings {
            red: LightPattern::ON,
            amber: LightPattern::ON,
            green: LightPattern::ON,
        }))
        .await;

    tick.next().await;
}
