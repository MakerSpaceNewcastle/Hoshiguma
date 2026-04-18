use crate::OnewireResources;
use defmt::info;
use ds18b20::{Ds18b20, Resolution};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_time::{Delay, Duration, Ticker, Timer};
use one_wire_bus::{Address, OneWire};

#[embassy_executor::task]
async fn task(r: OnewireResources) {
    let mut bus = {
        let pin = OutputOpenDrain::new(r.pin, Level::Low);
        OneWire::new(pin).unwrap()
    };

    // Scan bus and report discovered devices
    for device_address in bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let mut ticker = Ticker::every(Duration::from_secs(10));

    // let onboard_sensor = Ds18b20::new::<()>(Address(7949810265475014952)).unwrap();
    // let internal_ambient_sensor = Ds18b20::new::<()>(Address(6676982032140362024)).unwrap();
    // let coolant_pump_motor_sensor = Ds18b20::new::<()>(Address(8664048150377309736)).unwrap();
    // let reservoir_sensor = Ds18b20::new::<()>(Address(1945555040219935784)).unwrap();

    loop {
        ticker.next().await;

        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(Resolution::Bits12.max_measurement_time_millis() as u64).await;

        // let mut read_sensor = |sensor: &Ds18b20| -> OnewireTemperatureSensorReading {
        //     sensor
        //         .read_data(&mut bus, &mut Delay)
        //         .map(|r| r.temperature)
        //         .map_err(|_| ())
        // };

        // let readings = Temperatures {
        //     onboard: read_sensor(&onboard_sensor),
        //     internal_ambient: read_sensor(&internal_ambient_sensor),
        //     coolant_pump_motor: read_sensor(&coolant_pump_motor_sensor),
        //     reservoir: read_sensor(&reservoir_sensor),
        // };

        // info!("{}", readings);

        // TODO
    }
}

pub(crate) struct TemperatureSensors {}

impl TemperatureSensors {
    pub(crate) fn new(spawner: &Spawner, r: OnewireResources) -> Self {
        spawner.must_spawn(task(r));
        Self {}
    }

    // pub(crate) async fn get(&self) -> Temperatures {
    // }
}
