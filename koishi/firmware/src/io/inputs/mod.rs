pub(crate) mod gpio_debug;
pub(crate) mod gpio_isolated;

use telemetry_protocols::koishi::Inputs;

pub(crate) trait ReadInputs {
    fn read(&self) -> Inputs;
}
