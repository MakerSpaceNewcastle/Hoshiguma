use crate::{
    OnewireResources,
    devices::temperature::{
        TEMPERATURE_SENSOR_READING, onewire_sensor_to_named_temperature_sensor,
    },
};
use defmt::{info, warn};
use embassy_rp::{
    bind_interrupts,
    peripherals::PIO1,
    pio::{InterruptHandler, Pio},
    pio_programs::onewire::{PioOneWire, PioOneWireProgram, PioOneWireSearch},
};
use embassy_time::{Duration, Ticker, Timer};
use heapless::Vec;
use hoshiguma_api::{OnewireTemperatureSensorReading, OnewireTemperatureSensorReadings};

bind_interrupts!(struct Irqs {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: OnewireResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("owb temperature sensors").await;

    const CRC: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_MAXIM_DOW);

    let mut pio = Pio::new(r.pio, Irqs);

    let prg = PioOneWireProgram::new(&mut pio.common);
    let mut onewire = PioOneWire::new(&mut pio.common, pio.sm0, r.pin, &prg);

    let mut devices = Vec::<u64, { OnewireTemperatureSensorReadings::MAX_NUM_SENSORS }>::new();

    // Scan bus and discover devices
    {
        let mut search = PioOneWireSearch::new();
        for _ in 0..OnewireTemperatureSensorReadings::MAX_NUM_SENSORS {
            if !search.is_finished()
                && let Some(address) = search.next(&mut onewire).await
            {
                if CRC.checksum(&address.to_le_bytes()) == 0 {
                    info!("Found address: {:x}", address);
                    devices.push(address).unwrap();
                } else {
                    warn!("Found invalid address: {:x}", address);
                }
            }
        }
        if !search.is_finished() {
            warn!("Found max number of devices before search finished");
        }
        info!("Search done, found {} devices", devices.len());
    }

    let temperature_pub = TEMPERATURE_SENSOR_READING.publisher().unwrap();
    let mut tick = Ticker::every(Duration::from_secs(5));

    loop {
        tick.next().await;

        onewire.reset().await;
        // Skip rom and trigger conversion, we can trigger all devices on the bus immediately
        onewire.write_bytes(&[0xCC, 0x44]).await;

        // Allow time for the measurement to finish
        // Appropriate for 12 bit resolution
        Timer::after_millis(750).await;

        // Read all devices
        for device in &devices {
            onewire.reset().await;
            onewire.write_bytes(&[0x55]).await; // Match rom
            onewire.write_bytes(&device.to_le_bytes()).await;
            onewire.write_bytes(&[0xBE]).await; // Read scratchpad

            let mut data = [0; 9];
            onewire.read_bytes(&mut data).await;
            let reading = if CRC.checksum(&data) == 0 {
                let temp = ((data[1] as i16) << 8 | data[0] as i16) as f32 / 16.;
                info!("Read device {:x}: {} deg C", device, temp);
                Ok(temp)
            } else {
                warn!("Reading device {:x} failed", device);
                Err(())
            };

            let reading = OnewireTemperatureSensorReading {
                address: *device,
                reading,
            };
            let reading = onewire_sensor_to_named_temperature_sensor(reading);

            temperature_pub.publish(reading).await;
        }
    }
}
