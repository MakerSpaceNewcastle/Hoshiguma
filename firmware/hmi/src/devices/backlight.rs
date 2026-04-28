use embassy_time::Timer;
use peek_o_display_bsp::embassy_rp::gpio::Output;

#[embassy_executor::task]
pub(crate) async fn task(mut backlight: Output<'static>) {
    // TODO
    loop {
        backlight.toggle();
        Timer::after_secs(1).await;
    }
}
