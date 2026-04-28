use defmt::info;
use peek_o_display_bsp::{embassy_rp::gpio::Input, touch::Touch};

#[embassy_executor::task]
pub(crate) async fn task(mut touch: Touch, mut irq: Input<'static>) {
    touch.read();

    loop {
        irq.wait_for_low().await;

        // TODO
        let measurement = touch.read();
        info!("touch={}", measurement);
    }
}
