use defmt::info;
use embassy_time::Timer;
use peek_o_display_bsp::touch::Touch;

#[embassy_executor::task]
pub(crate) async fn task(mut touch: Touch) {
    touch.read();

    // TODO
    loop {
        let measurement = touch.read();
        info!("Touch measurement: {:?}", measurement);
        Timer::after_secs(1).await;
    }
}
