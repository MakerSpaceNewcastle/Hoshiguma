use super::SensorReadAndUpdate;
use crate::retry::retry;
use ds18b20::Ds18b20;
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use log::{error, info};
use one_wire_bus::OneWire;
use satori_state::sensors::{SensorError, SensorReading, SensorValue};

pub(crate) struct TemperatureSensors<P, E, D>
where
    P: InputPin<Error = E> + OutputPin<Error = E>,
    E: std::fmt::Debug,
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
            u64::from_str_radix($address, 16).expect(""),
        ))
        .expect("temperature sensor should be created from address")
    };
}

impl<
        P: InputPin<Error = E> + OutputPin<Error = E>,
        E: std::fmt::Debug,
        D: DelayUs<u16> + DelayMs<u16>,
    > TemperatureSensors<P, E, D>
{
    pub(crate) fn new(bus: OneWire<P>, delay: D) -> Self {
        Self {
            bus,
            delay,

            radiator_top: dallas_temperature_sensor!("9E3CE1E3803B3A28"), //TODO
            radiator_bottom: dallas_temperature_sensor!("9E3CE1E3803B3A28"), //TODO

            coolant_pump_case: dallas_temperature_sensor!("783CE1E3801EA628"),

            coolant_flow: dallas_temperature_sensor!("703CE1E380A2E828"),
            coolant_return: dallas_temperature_sensor!("523CE1E380B9B828"),

            laser_chamber_ambient: dallas_temperature_sensor!("8F3C53F649ABE528"),
            electronics_bay_ambient: dallas_temperature_sensor!("393CE1E3807F7528"),
            room_ambient: dallas_temperature_sensor!("F1561D5409646128"),
        }
    }

    pub(crate) fn debug_scan(&mut self) {
        // TODO: better error handling

        info!("Scanning one wire bus");
        ds18b20::start_simultaneous_temp_measurement(&mut self.bus, &mut self.delay).unwrap();

        let mut search_state = None;
        loop {
            match retry::<5, _, _>(|| {
                self.bus
                    .device_search(search_state.as_ref(), false, &mut self.delay)
            }) {
                Ok(Some((device_address, state))) => {
                    search_state = Some(state);

                    info!("Found device at address {:?}", device_address);

                    if device_address.family_code() == ds18b20::FAMILY_CODE {
                        let sensor = Ds18b20::new::<E>(device_address).unwrap();

                        if let Ok(sensor_data) =
                            retry::<5, _, _>(|| sensor.read_data(&mut self.bus, &mut self.delay))
                        {
                            info!(
                                "Found DS18B20 at address {:?} with temperature {}°C",
                                device_address, sensor_data.temperature
                            );
                        } else {
                            error!("Failed to read sensor at {:?}", device_address);
                        }
                    }
                }
                Err(e) => {
                    error!("fuck {:?}", e);
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn read_sensor(
        bus: &mut OneWire<P>,
        delay: &mut D,
        sensor: &Ds18b20,
        name: &str,
    ) -> SensorReading<f32> {
        info!("Reading {name}...");
        let sensor_data = retry::<5, _, _>(|| sensor.read_data(bus, delay))
            .map_err(|_| SensorError::ReadFailed)?;

        let value = sensor_data.temperature;
        info!("Sensor {name} reads {value}°C");

        Ok(SensorValue::new(value))
    }
}

impl<
        P: InputPin<Error = E> + OutputPin<Error = E>,
        E: std::fmt::Debug,
        D: DelayUs<u16> + DelayMs<u16>,
    > SensorReadAndUpdate for TemperatureSensors<P, E, D>
{
    fn read(&mut self) {
        info!("Starting temperature measurement");

        if let Err(e) = ds18b20::start_simultaneous_temp_measurement(&mut self.bus, &mut self.delay)
        {
            error!("Failed to start temperature measurement ({:?})", e);
            return;
        }

        let _radiator_top = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.radiator_top,
            "radiator_top",
        );

        let _radiator_bottom = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.radiator_bottom,
            "radiator_bottom",
        );

        let _coolant_pump_case = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.coolant_pump_case,
            "coolant_pump_case",
        );

        let _coolant_flow = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.coolant_flow,
            "coolant_flow",
        );

        let _coolant_return = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.coolant_return,
            "coolant_return",
        );

        let _laser_chamber_ambient = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.laser_chamber_ambient,
            "laser_chamber_ambient",
        );

        let _electronics_bay_ambient = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.electronics_bay_ambient,
            "electronics_bay_ambient",
        );

        let _room_ambient = Self::read_sensor(
            &mut self.bus,
            &mut self.delay,
            &self.room_ambient,
            "room_ambient",
        );

        // TODO
    }
}
