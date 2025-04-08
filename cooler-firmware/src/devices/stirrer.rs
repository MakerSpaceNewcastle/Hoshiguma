use crate::StirrerResources;
use embassy_rp::gpio::{Level, Output};
use hoshiguma_protocol::cooler::types::StirrerState;

pub(crate) struct Stirrer {
    output: Output<'static>,
}

impl Stirrer {
    pub(crate) fn new(r: StirrerResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self { output }
    }

    pub(crate) fn set(&mut self, state: StirrerState) {
        self.output.set_level(match state {
            StirrerState::Idle => Level::Low,
            StirrerState::Run => Level::High,
        });
    }

    pub(crate) fn get(&mut self) -> StirrerState {
        match self.output.get_output_level() {
            Level::Low => StirrerState::Idle,
            Level::High => StirrerState::Run,
        }
    }
}
