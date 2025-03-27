use crate::{rpc::report_event, OnewireResources};
use defmt::info;
use ds18b20::{Ds18b20, Resolution};
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_protocol::{
    cooler::{
        event::{EventKind, ObservationEvent},
        types::Temperatures,
    },
    types::TemperatureReading,
};
use one_wire_bus::{Address, OneWire};

pub(crate) trait TemperaturesExt {
    fn overall_result(&self) -> Result<(), ()>;
}

impl TemperaturesExt for Temperatures {
    fn overall_result(&self) -> Result<(), ()> {
        let sensors = [
            &self.onboard,
            &self.coolant_flow,
            &self.coolant_mid,
            &self.coolant_return,
            &self.heat_exchange_fluid,
            &self.heat_exchanger_loop,
        ];

        let any_error = sensors.iter().any(|i| i.is_err());

        if any_error {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub(crate) static TEMPERATURES_READ: Watch<CriticalSectionRawMutex, Temperatures, 5> = Watch::new();

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

    let onboard_sensor = Ds18b20::new::<()>(Address(17628307574231425320)).unwrap();
    let coolant_flow_sensor = Ds18b20::new::<()>(Address(8087587398082553896)).unwrap();
    let coolant_mid_sensor = Ds18b20::new::<()>(Address(8087587398082553896)).unwrap();
    let coolant_return_sensor = Ds18b20::new::<()>(Address(5925859576946210856)).unwrap();
    let heat_exchange_fluid_sensor = Ds18b20::new::<()>(Address(5925859576946210856)).unwrap();
    let heat_exchanger_loop_sensor = Ds18b20::new::<()>(Address(5925859576946210856)).unwrap();

    loop {
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

        // Telemetry reporting
        report_event(EventKind::Observation(ObservationEvent::Temperatures(
            readings.clone(),
        )))
        .await;

        tx.send(readings);

        ticker.next().await;
    }
}
