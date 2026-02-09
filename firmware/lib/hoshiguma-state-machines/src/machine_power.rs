use defmt::info;
use hoshiguma_api::{DesiredMachinePower, InterlockAction, hmi::AccessControlState};
use hoshiguma_common::changed::ObservedValue;

crate::state_machine!(InputMessage, OutputMessage, State, 8);

pub enum InputMessage {
    AccessControlState(AccessControlState),
    InterlockAction(InterlockAction),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    Power(DesiredMachinePower),
}

pub struct State {
    access_control: AccessControlState,
    interlock: InterlockAction,

    output_power: ObservedValue<DesiredMachinePower>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            access_control: AccessControlState::Denied,
            interlock: InterlockAction::Shutdown,

            output_power: ObservedValue::default(),
        }
    }
}

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match self.input_channel.receive().await {
                InputMessage::AccessControlState(state) => {
                    self.state.access_control = state;
                }
                InputMessage::InterlockAction(state) => {
                    self.state.interlock = state;
                }
            }

            let power = if self.state.access_control == AccessControlState::Granted
                && self.state.interlock != InterlockAction::Shutdown
            {
                DesiredMachinePower::On
            } else {
                DesiredMachinePower::Off
            };
            info!(
                "access control {} + interlock {} = desired power {}",
                self.state.access_control, self.state.interlock, power
            );

            self.state
                .output_power
                .update_and_async(power, async |v| {
                    self.output_channel.send(OutputMessage::Power(v)).await;
                })
                .await;
        }
    }
}
