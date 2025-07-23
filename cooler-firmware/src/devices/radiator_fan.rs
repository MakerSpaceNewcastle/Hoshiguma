use crate::RadiatorFanResources;
use hoshiguma_protocol::cooler::types::RadiatorFanState;
use pico_plc_bsp::embassy_rp::gpio::{Level, Output};

pub(crate) struct RadiatorFan {
    output: Output<'static>,
}

impl RadiatorFan {
    pub(crate) fn new(r: RadiatorFanResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self { output }
    }

    pub(crate) fn set(&mut self, state: RadiatorFanState) {
        self.output.set_level(match state {
            RadiatorFanState::Idle => Level::Low,
            RadiatorFanState::Run => Level::High,
        });
    }

    pub(crate) fn get(&mut self) -> RadiatorFanState {
        match self.output.get_output_level() {
            Level::Low => RadiatorFanState::Idle,
            Level::High => RadiatorFanState::Run,
        }
    }
}
