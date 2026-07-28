use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use hoshiguma_api::{OnewireTemperatureSensorReading, TemperatureSensor, TemperatureSensorReading};

pub(crate) static TEMPERATURE_SENSOR_READING: PubSubChannel<
    CriticalSectionRawMutex,
    TemperatureSensorReading,
    16,
    2,
    2,
> = PubSubChannel::new();

pub(super) fn onewire_sensor_to_named_temperature_sensor(
    reading: OnewireTemperatureSensorReading,
) -> TemperatureSensorReading {
    match reading.address {
        18165758753492984104 => TemperatureSensorReading {
            sensor: TemperatureSensor::OrchastratorPcb,
            reading: reading.reading,
        },
        16165716828082168104 => TemperatureSensorReading {
            sensor: TemperatureSensor::CoolerPcb,
            reading: reading.reading,
        },
        2522015792501619496 => TemperatureSensorReading {
            sensor: TemperatureSensor::CoolantReservoir,
            reading: reading.reading,
        },
        _ => {
            info!("Unknown temperature sensor {}", reading.address);
            reading.into()
        }
    }
}
