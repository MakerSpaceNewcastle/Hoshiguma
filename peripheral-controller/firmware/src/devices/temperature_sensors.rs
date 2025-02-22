use crate::{telemetry::queue_telemetry_message, OnewireResources};
use defmt::{info, Format};
use ds18b20::{Ds18b20, Resolution};
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};
use one_wire_bus::{Address, OneWire};

pub(crate) type TemperatureReading = Result<f32, ()>;

#[derive(Clone, Format)]
pub(crate) struct TemperatureReadings {
    pub(crate) onboard: TemperatureReading,
    pub(crate) electronics_bay_top: TemperatureReading,

    pub(crate) laser_chamber: TemperatureReading,

    pub(crate) ambient: TemperatureReading,

    pub(crate) coolant_flow: TemperatureReading,
    pub(crate) coolant_return: TemperatureReading,

    pub(crate) coolant_resevoir_bottom: TemperatureReading,
    pub(crate) coolant_resevoir_top: TemperatureReading,

    pub(crate) coolant_pump: TemperatureReading,
}

impl From<&TemperatureReadings>
    for hoshiguma_telemetry_protocol::payload::observation::Temperatures
{
    fn from(value: &TemperatureReadings) -> Self {
        Self {
            onboard: value.onboard,
            electronics_bay_top: value.electronics_bay_top,
            laser_chamber: value.laser_chamber,
            ambient: value.ambient,
            coolant_flow: value.coolant_flow,
            coolant_return: value.coolant_return,
            coolant_resevoir_bottom: value.coolant_resevoir_bottom,
            coolant_resevoir_top: value.coolant_resevoir_top,
            coolant_pump: value.coolant_pump,
        }
    }
}

impl TemperatureReadings {
    pub(crate) fn overall_result(&self) -> Result<(), ()> {
        let sensors = [
            &self.onboard,
            &self.electronics_bay_top,
            &self.ambient,
            &self.coolant_flow,
            &self.coolant_return,
            &self.coolant_resevoir_bottom,
            &self.coolant_resevoir_top,
            &self.coolant_pump,
        ];

        let any_error = sensors.iter().any(|i| i.is_err());

        if any_error {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub(crate) static TEMPERATURES_READ: Watch<CriticalSectionRawMutex, TemperatureReadings, 5> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: OnewireResources) {
    let mut bus = {
        let pin = OutputOpenDrain::new(r.pin, Level::Low);
        OneWire::new(pin).unwrap()
    };

    // Scan bus
    for device_address in bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let mut ticker = Ticker::every(Duration::from_secs(10));

    let tx = TEMPERATURES_READ.sender();

    let onboard_sensor = Ds18b20::new::<()>(Address(77134400158196008)).unwrap();
    let electronics_bay_top = Ds18b20::new::<()>(Address(17305478839918682408)).unwrap();
    let laser_chamber_sensor = Ds18b20::new::<()>(Address(10321216763289396520)).unwrap();
    let ambient_sensor = Ds18b20::new::<()>(Address(17390119257909780776)).unwrap();
    let coolant_flow_sensor = Ds18b20::new::<()>(Address(8087587398082553896)).unwrap();
    let coolant_return_sensor = Ds18b20::new::<()>(Address(5925859576946210856)).unwrap();
    let coolant_resevoir_top_sensor = Ds18b20::new::<()>(Address(953885588342016040)).unwrap();
    let coolant_resevoir_bottom_sensor = Ds18b20::new::<()>(Address(10753505152894955560)).unwrap();
    let coolant_pump_sensor = Ds18b20::new::<()>(Address(8664048150377309736)).unwrap();

    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut read_sensor = |sensor: &Ds18b20| -> TemperatureReading {
            sensor
                .read_data(&mut bus, &mut Delay)
                .map(|r| r.temperature)
                .map_err(|_| ())
        };

        let readings = TemperatureReadings {
            onboard: read_sensor(&onboard_sensor),
            electronics_bay_top: read_sensor(&electronics_bay_top),
            laser_chamber: read_sensor(&laser_chamber_sensor),
            ambient: read_sensor(&ambient_sensor),
            coolant_flow: read_sensor(&coolant_flow_sensor),
            coolant_return: read_sensor(&coolant_return_sensor),
            coolant_resevoir_top: read_sensor(&coolant_resevoir_top_sensor),
            coolant_resevoir_bottom: read_sensor(&coolant_resevoir_bottom_sensor),
            coolant_pump: read_sensor(&coolant_pump_sensor),
        };

        queue_telemetry_message(Payload::Observation(ObservationPayload::Temperatures(
            (&readings).into(),
        )))
        .await;

        tx.send(readings);

        ticker.next().await;
    }
}
