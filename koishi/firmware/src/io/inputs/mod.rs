pub(crate) mod gpio_debug;
pub(crate) mod gpio_isolated;

use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) enum ExtractionMode {
    Normal,
    Run,
}

#[derive(uDebug, PartialEq)]
pub(crate) struct Inputs {
    pub doors_closed: bool,
    pub cooling_ok: bool,
    pub machine_running: bool,
    pub air_pump_demand: bool,
    pub extraction_mode: ExtractionMode,
}

pub(crate) trait ReadInputs {
    fn read(&self) -> Inputs;
}
