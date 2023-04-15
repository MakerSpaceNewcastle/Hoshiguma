use super::{ExtractionMode, Inputs, ReadInputs};
use crate::unwrap_simple::UnwrapSimple;
use embedded_hal::digital::v2::InputPin;

pub(crate) struct GpioIsolatedInputs<A: InputPin, B: InputPin, C: InputPin, D: InputPin> {
    pub door_switches: A,
    pub extraction_mode_switch: B,
    pub machine_run_status: C,
    pub air_pump_demand: D,
}

#[macro_export]
macro_rules! gpio_isolated_inputs {
    ($pins:expr) => {
        $crate::io::inputs::gpio_isolated::GpioIsolatedInputs {
            door_switches: $pins.in1.into_floating_input(),
            extraction_mode_switch: $pins.in2.into_floating_input(),
            machine_run_status: $pins.in4.into_floating_input(),
            air_pump_demand: $pins.in5.into_floating_input(),
        }
    };
}

impl<A: InputPin, B: InputPin, C: InputPin, D: InputPin> ReadInputs
    for GpioIsolatedInputs<A, B, C, D>
{
    fn read(&self) -> Inputs {
        Inputs {
            doors_closed: self.door_switches.is_high().unwrap_simple(),
            cooling_ok: true,
            machine_running: self.machine_run_status.is_high().unwrap_simple(),
            air_pump_demand: self.air_pump_demand.is_high().unwrap_simple(),
            extraction_mode: if self.extraction_mode_switch.is_high().unwrap_simple() {
                ExtractionMode::Run
            } else {
                ExtractionMode::Normal
            },
        }
    }
}
