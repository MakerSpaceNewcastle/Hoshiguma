use super::Sdp810Resources;
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

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
