use embedded_hal_p2::blocking::delay::{DelayMs, DelayUs};
use hoshiguma_foundational_data::satori::Temperatures;
use onewire::{Device, OneWire, DS18B20};

macro_rules! dallas_temperature_sensor {
    ( $address:expr ) => {{
        let device = Device::from_str($address).unwrap();
        let sensor = DS18B20::new::<E>(device).unwrap();
        sensor
    }};
}

macro_rules! read_temperature_sensor {
    ($self: expr, $sensor: expr) => {{
        match $sensor.measure_temperature(&mut $self.bus, &mut $self.delay) {
            Ok(resolution) => {
                $self.delay.delay_ms(resolution.time_ms());

                match $sensor.read_temperature(&mut $self.bus, &mut $self.delay) {
                    Ok(result) => Some(result as f32),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }};
}

pub(crate) struct TemperatureSensors<'a, E, D>
where
    E: core::fmt::Debug,
    D: DelayMs<u16> + DelayUs<u16>,
{
    bus: OneWire<'a, E>,
    delay: D,

    coolant_radiator_upper: DS18B20,
    coolant_radiator_lower: DS18B20,

    coolant_pump_case: DS18B20,

    coolant_flow: DS18B20,
    coolant_return: DS18B20,

    laser_chamber_ambient: DS18B20,
    electronics_bay_ambient: DS18B20,
    room_ambient: DS18B20,
}

impl<'a, E, D> TemperatureSensors<'a, E, D>
where
    E: core::fmt::Debug,
    D: DelayMs<u16> + DelayUs<u16>,
{
    pub(crate) fn new(bus: OneWire<'a, E>, delay: D) -> Self {
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

    pub(crate) fn read(&mut self) -> Temperatures {
        Temperatures {
            coolant_flow: read_temperature_sensor!(self, self.coolant_flow),
            coolant_return: read_temperature_sensor!(self, self.coolant_return),
            coolant_resevoir_upper: read_temperature_sensor!(self, self.coolant_radiator_upper),
            coolant_resevoir_lower: read_temperature_sensor!(self, self.coolant_radiator_lower),
            coolant_pump: read_temperature_sensor!(self, self.coolant_pump_case),
            room_ambient: read_temperature_sensor!(self, self.room_ambient),
            laser_bay: read_temperature_sensor!(self, self.laser_chamber_ambient),
            electronics_bay: read_temperature_sensor!(self, self.electronics_bay_ambient),
        }
    }
}
