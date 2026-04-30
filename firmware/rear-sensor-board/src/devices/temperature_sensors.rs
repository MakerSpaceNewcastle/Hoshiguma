use crate::{OnewireResources, api::NUM_LISTENERS};
use defmt::{Format, info, warn};
use embassy_rp::{
    bind_interrupts,
    peripherals::PIO1,
    pio::{InterruptHandler, Pio},
    pio_programs::onewire::{PioOneWire, PioOneWireProgram, PioOneWireSearch},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, with_timeout};
use heapless::Vec;
use hoshiguma_api::{OnewireTemperatureSensorReading, OnewireTemperatureSensorReadings};
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) struct Request;
#[derive(Clone, Format)]
pub(crate) struct Response(OnewireTemperatureSensorReadings);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait TemperatureInterfaceChannel {
    async fn get(&mut self) -> Result<OnewireTemperatureSensorReadings, ()>;
}

impl TemperatureInterfaceChannel for TheirChannelSide {
    async fn get(&mut self) -> Result<OnewireTemperatureSensorReadings, ()> {
        self.send(Request).await;

        match with_timeout(Duration::from_millis(1200), self.receive()).await {
            Ok(response) => Ok(response.0),
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

bind_interrupts!(struct Irqs {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: OnewireResources, comm: [MyChannelSide; NUM_LISTENERS]) {
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
                    info!("Found addres: {:x}", address);
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

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());
        let (_, idx) = embassy_futures::select::select_array(rx_futures).await;

        onewire.reset().await;
        // Skip rom and trigger conversion, we can trigger all devices on the bus immediately
        onewire.write_bytes(&[0xCC, 0x44]).await;

        // Allow time for the measurement to finish
        // Appropriate for 12 bit resolution
        Timer::after_millis(750).await;

        // Read all devices
        let mut readings = OnewireTemperatureSensorReadings::default();
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

            readings
                .push(OnewireTemperatureSensorReading::new(*device, reading))
                .unwrap();
        }

        comm[idx].send(Response(readings)).await;
    }
}
