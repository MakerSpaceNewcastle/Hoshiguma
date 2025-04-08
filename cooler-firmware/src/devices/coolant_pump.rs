use crate::CoolantPumpResources;
use embassy_rp::gpio::{Level, Output};
use hoshiguma_protocol::cooler::types::CoolantPumpState;

pub(crate) struct CoolantPump {
    output: Output<'static>,
}

impl CoolantPump {
    pub(crate) fn new(r: CoolantPumpResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self { output }
    }

    pub(crate) fn set(&mut self, state: CoolantPumpState) {
        self.output.set_level(match state {
            CoolantPumpState::Idle => Level::Low,
            CoolantPumpState::Run => Level::High,
        });
    }

    pub(crate) fn get(&mut self) -> CoolantPumpState {
        match self.output.get_output_level() {
            Level::Low => CoolantPumpState::Idle,
            Level::High => CoolantPumpState::Run,
        }
    }
}
