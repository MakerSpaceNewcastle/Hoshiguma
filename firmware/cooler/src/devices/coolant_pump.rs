use crate::CoolantPumpResources;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use hoshiguma_api::cooler::CoolantPumpState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSide};

pub(crate) type CoolantPumpChannel = BiDirectionalChannel<CriticalSectionRawMutex, (), (), 8, 1, 1>;
type Us = <CoolantPumpChannel as BiDirectionalChannelSide>::SideA;

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
