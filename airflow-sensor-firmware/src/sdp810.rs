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
    config.frequency = 400_000;
    config.sda_pullup = true;
    config.scl_pullup = true;
    let mut i2c = I2c::new_async(r.i2c, r.scl_pin, r.sda_pin, Irqs, config);

    Timer::after_millis(500).await;

    sensirion_i2c::i2c_async::write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_1)
        .await
        .unwrap();
    sensirion_i2c::i2c_async::write_command_u16(&mut i2c, DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_2)
        .await
        .unwrap();
    let mut buff = [0u8; 20];
    i2c.read_async(DEVICE_ADDRESS, &mut buff).await.unwrap();
    info!("got bytes: {}", buff);

    i2c.write_async(DEVICE_ADDRESS, CMD_CONT_MASS_FLOW_AVG.to_be_bytes())
        .await
        .unwrap();

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
