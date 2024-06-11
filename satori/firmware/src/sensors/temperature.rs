use ds18b20::{Ds18b20, Resolution};
use embedded_hal_p2::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use one_wire_bus::OneWire;

pub(crate) struct TemperatureSensors<P, E, D>
where
    P: InputPin<Error = E> + OutputPin<Error = E>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    bus: OneWire<P>,
    delay: D,

    radiator_top: Ds18b20,
    radiator_bottom: Ds18b20,

    coolant_pump_case: Ds18b20,

    coolant_flow: Ds18b20,
    coolant_return: Ds18b20,

    laser_chamber_ambient: Ds18b20,
    electronics_bay_ambient: Ds18b20,
    room_ambient: Ds18b20,
}

macro_rules! dallas_temperature_sensor {
    ( $address:expr ) => {
        Ds18b20::new::<()>(one_wire_bus::Address(
            u64::from_str_radix($address, 16).expect("address should be valid hex"),
        ))
        .expect("temperature sensor should be created from address")
    };
}

impl<P, E, D> TemperatureSensors<P, E, D>
where
    P: InputPin<Error = E> + OutputPin<Error = E>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    pub(crate) fn new(bus: OneWire<P>, delay: D) -> Self {
        Self {
            bus,
            delay,
            radiator_top: dallas_temperature_sensor!(""),
            radiator_bottom: dallas_temperature_sensor!(""),
            coolant_pump_case: dallas_temperature_sensor!(""),
            coolant_flow: dallas_temperature_sensor!(""),
            coolant_return: dallas_temperature_sensor!(""),
            laser_chamber_ambient: dallas_temperature_sensor!(""),
            electronics_bay_ambient: dallas_temperature_sensor!(""),
            room_ambient: dallas_temperature_sensor!(""),
        }
    }

    fn begin_measurement(&mut self) -> Result<(), ()> {
        match ds18b20::start_simultaneous_temp_measurement(&mut self.bus, &mut self.delay) {
            Ok(_) => {
                Resolution::Bits12.delay_for_measurement_time(&mut self.delay);
                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}
