use super::Sdp810Resources;
use defmt::info;
use embassy_rp::{
    bind_interrupts,
    i2c::{Config, I2c, InterruptHandler},
    peripherals::I2C1,
};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub(super) async fn task(r: Sdp810Resources) -> ! {
    let mut i2c = I2c::new_async(r.i2c, r.scl_pin, r.sda_pin, Irqs, Config::default());

    i2c.write_async(DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_1.to_be_bytes())
        .await
        .unwrap();
    i2c.write_async(DEVICE_ADDRESS, CMD_READ_PRODUCT_ID_2.to_be_bytes())
        .await
        .unwrap();
    let mut buff = [0u8; 20];
    i2c.read_async(DEVICE_ADDRESS, &mut buff).await.unwrap();
    info!("got bytes: {}", buff);

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

const DEVICE_ADDRESS: u8 = 0x25;

/// Command to start continuous measurement, using mass flow temperature compensation, averaging up
/// to point of readout.
const CMD_CONT_MASS_FLOW_AVG: u32 = 0x3603;

const CMD_READ_PRODUCT_ID_1: u32 = 0x367C;
const CMD_READ_PRODUCT_ID_2: u32 = 0xE102;
