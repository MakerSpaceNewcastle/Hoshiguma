use crate::CompressorResources;
use embassy_rp::gpio::{Level, Output};
use hoshiguma_core::accessories::cooler::types::CompressorState;

pub(crate) struct Compressor {
    output: Output<'static>,
}

impl Compressor {
    pub(crate) fn new(r: CompressorResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self { output }
    }

    pub(crate) fn set(&mut self, state: CompressorState) {
        self.output.set_level(match state {
            CompressorState::Idle => Level::Low,
            CompressorState::Run => Level::High,
        });
    }

    pub(crate) fn get(&mut self) -> CompressorState {
        match self.output.get_output_level() {
            Level::Low => CompressorState::Idle,
            Level::High => CompressorState::Run,
        }
    }
}
