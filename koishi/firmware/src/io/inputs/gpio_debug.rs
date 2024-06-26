use super::ReadInputs;
use crate::unwrap_simple::UnwrapSimple;
use embedded_hal::digital::v2::InputPin;
use hoshiguma_foundational_data::koishi::{ExtractionMode, Inputs};

#[allow(dead_code)]
pub(crate) struct GpioDebugInputs<A: InputPin, B: InputPin, C: InputPin, D: InputPin, E: InputPin> {
    pub door_switches: A,
    pub external_enable: B,

    pub machine_run_status: C,
    pub air_pump_demand: D,

    pub extraction_mode_switch: E,
}

#[macro_export]
macro_rules! gpio_debug_inputs {
    ($pins:expr) => {
        $crate::io::inputs::gpio_debug::GpioDebugInputs {
            door_switches: $pins.in1.into_pull_up_input(),
            external_enable: $pins.in6.into_pull_up_input(),

            machine_run_status: $pins.in4.into_pull_up_input(),
            air_pump_demand: $pins.in5.into_pull_up_input(),

            extraction_mode_switch: $pins.in2.into_pull_up_input(),
        }
    };
}

impl<A: InputPin, B: InputPin, C: InputPin, D: InputPin, E: InputPin> ReadInputs
    for GpioDebugInputs<A, B, C, D, E>
{
    fn read(&self) -> Inputs {
        Inputs {
            doors_closed: self.door_switches.is_low().unwrap_simple(),
            external_enable: self.external_enable.is_low().unwrap_simple(),
            machine_running: self.machine_run_status.is_low().unwrap_simple(),
            air_pump_demand: self.air_pump_demand.is_low().unwrap_simple(),
            extraction_mode: if self.extraction_mode_switch.is_low().unwrap_simple() {
                ExtractionMode::Run
            } else {
                ExtractionMode::Normal
            },
        }
    }
}
