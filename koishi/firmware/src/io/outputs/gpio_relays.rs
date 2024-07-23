use super::{Outputs, WriteOutputs};
use crate::unwrap_simple::UnwrapSimple;
use embedded_hal::digital::v2::OutputPin;
use hoshiguma_foundational_data::koishi::{AlarmState, StatusLight};

pub(crate) struct GpioRelayOutputs<
    A: OutputPin,
    B: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
    H: OutputPin,
> {
    pub air_pump: A,
    pub extraction_fan: B,
    pub laser_enable: D,
    pub controller_cooling_alarm: E,
    pub controller_machine_alarm: F,
    pub status_light_2: G,
    pub status_light_1: H,
}

#[macro_export]
macro_rules! gpio_relay_outputs_init_pin {
    ($thingy:expr) => {{
        let mut pin = $thingy.into_output();
        pin.set_high();
        pin
    }};
}

#[macro_export]
macro_rules! gpio_relay_outputs {
    ($pins:expr) => {
        $crate::io::outputs::gpio_relays::GpioRelayOutputs {
            air_pump: gpio_relay_outputs_init_pin!($pins.relay1),
            extraction_fan: gpio_relay_outputs_init_pin!($pins.relay2),
            laser_enable: gpio_relay_outputs_init_pin!($pins.relay4),
            controller_cooling_alarm: gpio_relay_outputs_init_pin!($pins.relay5),
            controller_machine_alarm: gpio_relay_outputs_init_pin!($pins.relay6),
            status_light_2: gpio_relay_outputs_init_pin!($pins.relay7),
            status_light_1: gpio_relay_outputs_init_pin!($pins.relay8),
        }
    };
}

impl<
        A: OutputPin,
        B: OutputPin,
        D: OutputPin,
        E: OutputPin,
        F: OutputPin,
        G: OutputPin,
        H: OutputPin,
    > WriteOutputs for GpioRelayOutputs<A, B, D, E, F, G, H>
{
    fn write(&mut self, outputs: &Outputs) {
        match outputs.controller_machine_alarm {
            AlarmState::Normal => self.controller_machine_alarm.set_low(),
            AlarmState::Alarm => self.controller_machine_alarm.set_high(),
        }
        .unwrap_simple();

        match outputs.controller_cooling_alarm {
            AlarmState::Normal => self.controller_cooling_alarm.set_low(),
            AlarmState::Alarm => self.controller_cooling_alarm.set_high(),
        }
        .unwrap_simple();

        match outputs.laser_enable {
            true => self.laser_enable.set_low(),
            false => self.laser_enable.set_high(),
        }
        .unwrap_simple();

        match outputs.status_light {
            StatusLight::Red => {
                self.status_light_1.set_high().unwrap_simple();
                self.status_light_2.set_high().unwrap_simple();
            }
            StatusLight::Amber => {
                self.status_light_1.set_high().unwrap_simple();
                self.status_light_2.set_low().unwrap_simple();
            }
            StatusLight::Green => {
                self.status_light_1.set_low().unwrap_simple();
                self.status_light_2.set_high().unwrap_simple();
            }
        }

        match outputs.air_pump {
            true => self.air_pump.set_low(),
            false => self.air_pump.set_high(),
        }
        .unwrap_simple();

        match outputs.extractor_fan {
            true => self.extraction_fan.set_low(),
            false => self.extraction_fan.set_high(),
        }
        .unwrap_simple();
    }
}
