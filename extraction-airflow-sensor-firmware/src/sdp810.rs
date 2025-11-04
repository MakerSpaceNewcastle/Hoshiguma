use super::Sdp810Resources;
use defmt::{debug, info};
use embassy_rp::{
    bind_interrupts,
    i2c::{Config, I2c, InterruptHandler},
    peripherals::I2C1,
};
use embassy_time::{Duration, Ticker, Timer};
use sensirion_i2c::i2c_async::{read_words_with_crc, write_command_u16, write_command_u8};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub(super) async fn task(r: Sdp810Resources) -> ! {
    let mut config = Config::default();
    config.frequency = 400_000;
    let mut i2c = I2c::new_async(r.i2c, r.scl_pin, r.sda_pin, Irqs, config);

    Timer::after_millis(100).await;

    // Soft reset the device
    write_command_u8(&mut i2c, 0x00, 0x06).await.unwrap();

    Timer::after_millis(100).await;

    // Get sensor product data
    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_1)
        .await
        .unwrap();
    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_2)
        .await
        .unwrap();

    let mut buff = [0u8; 18];
    read_words_with_crc(&mut i2c, DEVICE_ADDRESS, &mut buff).await;
    debug!("Got product ID bytes: {}", buff);

    write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_CONT_MASS_FLOW_AVG)
        .await
        .unwrap();

    let mut tick = Ticker::every(Duration::from_millis(1000));

    loop {
        let mut buffer = [0u8; 9];
        read_words_with_crc(&mut i2c, DEVICE_ADDRESS, &mut buffer).await;
        debug!("Got measurement bytes: {}", buffer);

        let pressure = i16::from(buffer[0]) << 8 | i16::from(buffer[1]);
        let temperature = i16::from(buffer[3]) << 8 | i16::from(buffer[4]);
        let pressure_scale = i16::from(buffer[6]) << 8 | i16::from(buffer[7]);

        debug!("Pressure scale: {}", pressure_scale);

        let pressure = f32::from(pressure) / f32::from(pressure_scale);
        let temperature = f32::from(temperature) / TEMPERATURE_SCALE_FACTOR;

        info!("pressure: {} Pa", pressure);
        info!("temperature: {} C", temperature);

        tick.next().await;
    }
}

const DEVICE_ADDRESS: u8 = 0x25;

/// Command to start continuous measurement, using mass flow temperature compensation, averaging up
/// to point of readout.
const CMD_CONT_MASS_FLOW_AVG: u16 = 0x3603;

const CMD_READ_PRODUCT_ID_1: u16 = 0x367C;
const CMD_READ_PRODUCT_ID_2: u16 = 0xE102;

const TEMPERATURE_SCALE_FACTOR: f32 = 200.0f32;
