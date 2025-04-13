use crate::OnewireResources;
use core::cell::RefCell;
use defmt::{info, unwrap};
use ds18b20::{Ds18b20, Resolution};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_protocol::{cooler::types::Temperatures, types::TemperatureReading};
use one_wire_bus::{Address, OneWire};

static READING: Mutex<CriticalSectionRawMutex, RefCell<Temperatures>> =
    Mutex::new(RefCell::new(Temperatures {
        onboard: Err(()),
        coolant_flow: Err(()),
        coolant_mid: Err(()),
        coolant_return: Err(()),
        heat_exchange_fluid: Err(()),
        heat_exchanger_loop: Err(()),
    }));

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

    let onboard_sensor = Ds18b20::new::<()>(Address(7949810265475014952)).unwrap();

    let heat_exchange_fluid_sensor = Ds18b20::new::<()>(Address(8217700785146991144)).unwrap();

    let coolant_flow_sensor = Ds18b20::new::<()>(Address(2766436500490182952)).unwrap();
    let heat_exchanger_loop_sensor = Ds18b20::new::<()>(Address(16939228239646449960)).unwrap();
    let coolant_mid_sensor = Ds18b20::new::<()>(Address(9708086592746578216)).unwrap();
    let coolant_return_sensor = Ds18b20::new::<()>(Address(2989910039812399400)).unwrap();

    loop {
        ticker.next().await;

        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut read_sensor = |sensor: &Ds18b20| -> TemperatureReading {
            sensor
                .read_data(&mut bus, &mut Delay)
                .map(|r| r.temperature)
                .map_err(|_| ())
        };

        let readings = Temperatures {
            onboard: read_sensor(&onboard_sensor),
            coolant_flow: read_sensor(&coolant_flow_sensor),
            coolant_mid: read_sensor(&coolant_mid_sensor),
            coolant_return: read_sensor(&coolant_return_sensor),
            heat_exchange_fluid: read_sensor(&heat_exchange_fluid_sensor),
            heat_exchanger_loop: read_sensor(&heat_exchanger_loop_sensor),
        };

        info!("{}", readings);

        READING.lock().await.replace(readings);
    }
}

pub(crate) struct TemperatureSensors {}

impl TemperatureSensors {
    pub(crate) fn new(spawner: &Spawner, r: OnewireResources) -> Self {
        unwrap!(spawner.spawn(task(r)));
        Self {}
    }

    pub(crate) async fn get(&self) -> Temperatures {
        READING.lock().await.borrow().clone()
    }
}
