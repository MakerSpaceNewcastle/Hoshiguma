use crate::{Sdp810Resources, api::NUM_LISTENERS};
use defmt::{Format, debug, info, warn};
use embassy_rp::{
    bind_interrupts,
    i2c::{Config, I2c, InterruptHandler},
    peripherals::I2C0,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, with_timeout};
use hoshiguma_api::{AirflowSensorMeasurement, AirflowSensorMeasurementInner};
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};
use sensirion_i2c::i2c_async::{read_words_with_crc, write_command_u8, write_command_u16};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) struct Request;
#[derive(Clone, Format)]
pub(crate) struct Response(AirflowSensorMeasurement);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait AirflowSensorInterfaceChannel {
    async fn get(&mut self) -> Result<AirflowSensorMeasurement, ()>;
}

impl AirflowSensorInterfaceChannel for TheirChannelSide {
    async fn get(&mut self) -> Result<AirflowSensorMeasurement, ()> {
        self.send(Request).await;

        match with_timeout(Duration::from_millis(500), self.receive()).await {
            Ok(response) => Ok(response.0),
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

const DEVICE_ADDRESS: u8 = 0x25;

/// Command to start continuous measurement, using mass flow temperature compensation, averaging up
/// to point of readout.
const CMD_CONT_MASS_FLOW_AVG: u16 = 0x3603;

const CMD_READ_PRODUCT_ID_1: u16 = 0x367C;
const CMD_READ_PRODUCT_ID_2: u16 = 0xE102;

const TEMPERATURE_SCALE_FACTOR: f32 = 200.0f32;

#[embassy_executor::task]
pub(crate) async fn task(r: Sdp810Resources, comm: [MyChannelSide; NUM_LISTENERS]) {
    let mut config = Config::default();
    config.frequency = 400_000;
    let mut i2c = I2c::new_async(r.i2c, r.scl, r.sda, Irqs, config);

    Timer::after_millis(10).await;

    // Soft reset the device
    write_command_u8(&mut i2c, 0x00, 0x06).await.unwrap();

    Timer::after_millis(10).await;

    // Request sensor product data
    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_1)
        .await
        .unwrap();
    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_2)
        .await
        .unwrap();

    // Receive sensor product data
    let mut buff = [0u8; 18];
    read_words_with_crc(&mut i2c, DEVICE_ADDRESS, &mut buff)
        .await
        .map_err(|_| ())
        .unwrap();
    debug!("Got product ID bytes: {}", buff);

    // Start continuous measurement
    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_CONT_MASS_FLOW_AVG)
        .await
        .unwrap();

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());
        let (_, idx) = embassy_futures::select::select_array(rx_futures).await;

        let reading = {
            let mut buffer = [0u8; 9];
            match read_words_with_crc(&mut i2c, DEVICE_ADDRESS, &mut buffer).await {
                Ok(_) => {
                    debug!("Got measurement bytes: {}", buffer);

                    let pressure = i16::from(buffer[0]) << 8 | i16::from(buffer[1]);
                    let temperature = i16::from(buffer[3]) << 8 | i16::from(buffer[4]);
                    let pressure_scale = i16::from(buffer[6]) << 8 | i16::from(buffer[7]);

                    debug!("Pressure scale: {}", pressure_scale);

                    let pressure = f32::from(pressure) / f32::from(pressure_scale);
                    let temperature = f32::from(temperature) / TEMPERATURE_SCALE_FACTOR;

                    let measurement = AirflowSensorMeasurementInner {
                        differential_pressure: pressure,
                        temperature,
                    };
                    info!("{}", measurement);
                    Ok(measurement)
                }
                Err(_) => {
                    warn!("Failed to read sensor data");
                    Err(())
                }
            }
        };

        comm[idx].send(Response(reading)).await;
    }
}
