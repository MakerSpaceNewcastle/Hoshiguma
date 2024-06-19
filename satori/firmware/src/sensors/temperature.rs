use crate::read_temperature_sensor;
use ds18b20::{Ds18b20, Resolution};
use embedded_hal_p2::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use hoshiguma_foundational_data::satori::Temperatures;
use onewire::OneWire;

pub(crate) struct TemperatureSensors<P, E, D>
where
    P: InputPin<Error = E> + OutputPin<Error = E>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    bus: OneWire<P>,
    delay: D,

    coolant_radiator_upper: Ds18b20,
    coolant_radiator_lower: Ds18b20,

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
            u64::from_str_radix($address, 16).unwrap(),
        ))
        .unwrap()
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
            coolant_radiator_upper: dallas_temperature_sensor!("0D3CE1E3817D8828"),
            coolant_radiator_lower: dallas_temperature_sensor!("953C1FF648A2F028"),
            coolant_pump_case: dallas_temperature_sensor!("783CE1E3801EA628"),
            coolant_flow: dallas_temperature_sensor!("703CE1E380A2E828"),
            coolant_return: dallas_temperature_sensor!("523CE1E380B9B828"),
            laser_chamber_ambient: dallas_temperature_sensor!("8F3C53F649ABE528"),
            electronics_bay_ambient: dallas_temperature_sensor!("BAFC5B5509646128"),
            room_ambient: dallas_temperature_sensor!("F1561D5409646128"),
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

    pub(crate) fn read(&mut self) -> Temperatures {
        // TODO: see https://github.com/Rahix/avr-hal/issues/541
        // match self.begin_measurement() {
        //     Ok(_) => Temperatures {
        //         coolant_flow: read_temperature_sensor!(self, self.coolant_flow),
        //         coolant_return: read_temperature_sensor!(self, self.coolant_return),
        //         coolant_resevoir_upper: read_temperature_sensor!(self, self.coolant_radiator_upper),
        //         coolant_resevoir_lower: read_temperature_sensor!(self, self.coolant_radiator_lower),
        //         coolant_pump: read_temperature_sensor!(self, self.coolant_pump_case),
        //         room_ambient: read_temperature_sensor!(self, self.room_ambient),
        //         laser_bay: read_temperature_sensor!(self, self.laser_chamber_ambient),
        //         electronics_bay: read_temperature_sensor!(self, self.electronics_bay_ambient),
        //     },
        //     Err(_) => Temperatures::default(),
        // }
        Temperatures::default()
    }
}

#[macro_export]
macro_rules! read_temperature_sensor {
    ($self: expr, $sensor: expr) => {
        match $sensor.read_data(&mut $self.bus, &mut $self.delay) {
            Ok(r) => Some(r.temperature),
            Err(_) => None,
        }
    };
}
