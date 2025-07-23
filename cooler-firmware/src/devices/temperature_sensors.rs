use crate::OnewireResources;
use core::cell::RefCell;
use defmt::{info, unwrap};
use ds18b20::{Ds18b20, Resolution};
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_protocol::{cooler::types::Temperatures, types::TemperatureReading};
use one_wire_bus::{Address, OneWire};
use pico_plc_bsp::embassy_rp::gpio::{Level, OutputOpenDrain};

static READING: Mutex<CriticalSectionRawMutex, RefCell<Temperatures>> =
    Mutex::new(RefCell::new(Temperatures {
        onboard: Err(()),
        internal_ambient: Err(()),
        reservoir_evaporator_coil: Err(()),
        reservoir_left_side: Err(()),
        reservoir_right_side: Err(()),
        coolant_pump_motor: Err(()),
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
    let internal_ambient_sensor = Ds18b20::new::<()>(Address(6676982032140362024)).unwrap();
    let reservoir_evaporator_coil = Ds18b20::new::<()>(Address(14339461214714576936)).unwrap();
    let reservoir_left_side = Ds18b20::new::<()>(Address(9079256849946264616)).unwrap();
    let reservoir_right_side = Ds18b20::new::<()>(Address(9943947978400069416)).unwrap();
    let coolant_pump_motor = Ds18b20::new::<()>(Address(8664048150377309736)).unwrap();

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
            internal_ambient: read_sensor(&internal_ambient_sensor),
            reservoir_evaporator_coil: read_sensor(&reservoir_evaporator_coil),
            reservoir_left_side: read_sensor(&reservoir_left_side),
            reservoir_right_side: read_sensor(&reservoir_right_side),
            coolant_pump_motor: read_sensor(&coolant_pump_motor),
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
