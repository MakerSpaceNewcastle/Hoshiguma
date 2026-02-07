use super::TemperaturesExt;
use crate::{OnewireResources, telemetry::queue_telemetry_data_point};
use defmt::{info, warn};
use ds18b20::{Ds18b20, Resolution};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Delay, Duration, Ticker, Timer};
use hoshiguma_core::{
    telemetry::AsTelemetry,
    types::{MachineTemperatures, TemperatureReading},
};
use one_wire_bus::{Address, OneWire};
use pico_plc_bsp::embassy_rp::gpio::{Level, OutputOpenDrain};

impl TemperaturesExt for MachineTemperatures {
    fn any_failed_sensors(&self) -> bool {
        let sensors = [
            &self.onboard,
            &self.electronics_bay_top,
            &self.laser_chamber,
            &self.coolant_flow,
            &self.coolant_return,
        ];

        sensors.iter().any(|i| i.is_err())
    }
}

pub(crate) static TEMPERATURES_READ: Watch<CriticalSectionRawMutex, MachineTemperatures, 5> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: OnewireResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("temp sensors a").await;

    let mut bus = {
        let pin = OutputOpenDrain::new(r.pin, Level::Low);
        OneWire::new(pin).unwrap()
    };

    // Scan bus
    for device_address in bus.devices(false, &mut embassy_time::Delay) {
        match device_address {
            Ok(device_address) => {
                info!("Found one wire device at address: {}", device_address.0);
            }
            Err(_) => {
                warn!("Failed to read onewire device");
            }
        }
    }

    let mut ticker = Ticker::every(Duration::from_secs(10));

    let tx = TEMPERATURES_READ.sender();

    let onboard_sensor = Ds18b20::new::<()>(Address(17628307574231425320)).unwrap();
    let electronics_bay_top = Ds18b20::new::<()>(Address(17305478839918682408)).unwrap();
    let laser_chamber_sensor = Ds18b20::new::<()>(Address(10321216763289396520)).unwrap();
    let coolant_flow_sensor = Ds18b20::new::<()>(Address(8087587398082553896)).unwrap();
    let coolant_return_sensor = Ds18b20::new::<()>(Address(5925859576946210856)).unwrap();

    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut read_sensor = |sensor: &Ds18b20| -> TemperatureReading {
            sensor
                .read_data(&mut bus, &mut Delay)
                .map(|r| r.temperature)
                .map_err(|_| ())
        };

        let readings = MachineTemperatures {
            onboard: read_sensor(&onboard_sensor),
            electronics_bay_top: read_sensor(&electronics_bay_top),
            laser_chamber: read_sensor(&laser_chamber_sensor),
            coolant_flow: read_sensor(&coolant_flow_sensor),
            coolant_return: read_sensor(&coolant_return_sensor),
        };

        for dp in readings.telemetry() {
            queue_telemetry_data_point(dp);
        }

        tx.send(readings);

        ticker.next().await;
    }
}
