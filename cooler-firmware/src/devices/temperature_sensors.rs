use crate::{rpc::report_event, OnewireResources};
use defmt::info;
use ds18b20::{Ds18b20, Resolution};
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_protocol::{
    cooler::{
        event::{EventKind, ObservationEvent},
        types::Temperatures,
    },
    types::TemperatureReading,
};
use one_wire_bus::{Address, OneWire};

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
            readings,
        )))
        .await;

        ticker.next().await;
    }
}
