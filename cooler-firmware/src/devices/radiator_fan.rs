use crate::RadiatorFanResources;
use embassy_rp::gpio::{Level, Output};
use hoshiguma_protocol::cooler::types::RadiatorFanState;

pub(crate) struct RadiatorFan {
    output: Output<'static>,
}

impl RadiatorFan {
    pub(crate) fn new(r: RadiatorFanResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self { output }
    }

    pub(crate) fn set(&mut self, state: RadiatorFanState) {
        let level = match state {
            RadiatorFanState::Idle => Level::Low,
            RadiatorFanState::Run => Level::High,
        };

        self.output.set_level(level);
    }

    pub(crate) fn get(&mut self) -> RadiatorFanState {
        let level = self.output.get_output_level();

        match level {
            Level::Low => RadiatorFanState::Idle,
            Level::High => RadiatorFanState::Run,
        }
    }
}
