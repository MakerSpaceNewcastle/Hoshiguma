pub(crate) mod gpio_debug;
pub(crate) mod gpio_isolated;

use hoshiguma_foundational_data::koishi::Inputs;

pub(crate) trait ReadInputs {
    fn read(&self) -> Inputs;
}
