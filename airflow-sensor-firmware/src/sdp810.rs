use super::Sdp810Resources;
use defmt::info;
use embassy_rp::{
    bind_interrupts,
    i2c::{Config, I2c, InterruptHandler},
    peripherals::I2C1,
};
use embassy_time::Timer;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub(super) async fn task(r: Sdp810Resources) -> ! {
    let mut config = Config::default();
    config.frequency = 100_000;
    // let mut i2c = I2c::new_async(r.i2c, r.scl_pin, r.sda_pin, Irqs, config);
    let mut i2c = I2c::new_blocking(r.i2c, r.scl_pin, r.sda_pin, config);

    Timer::after_millis(500).await;

    sensirion_i2c::i2c::write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_1).unwrap();
    sensirion_i2c::i2c::write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_2).unwrap();

    let mut buff = [0u8; 18];
    sensirion_i2c::i2c::read_words_with_crc(&mut i2c, DEVICE_ADDRESS, &mut buff);
    info!("got bytes: {}", buff);

    loop {
        Timer::after_secs(1).await;
    }
}

const DEVICE_ADDRESS: u8 = 0x25;

/// Command to start continuous measurement, using mass flow temperature compensation, averaging up
/// to point of readout.
const CMD_CONT_MASS_FLOW_AVG: u16 = 0x3603;

const CMD_READ_PRODUCT_ID_1: u16 = 0x367C;
const CMD_READ_PRODUCT_ID_2: u16 = 0xE102;
