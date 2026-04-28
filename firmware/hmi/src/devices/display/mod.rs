use embassy_time::Timer;
use embedded_graphics::{
    pixelcolor::Rgb666,
    prelude::{DrawTarget, RgbColor},
};
use peek_o_display_bsp::display::Display;

#[embassy_executor::task]
pub(crate) async fn task(mut display: Display) {
    // TODO
    loop {
        display.clear(Rgb666::RED).unwrap();
        Timer::after_secs(2).await;
        display.clear(Rgb666::GREEN).unwrap();
        Timer::after_secs(2).await;
        display.clear(Rgb666::BLUE).unwrap();
        Timer::after_secs(2).await;
        display.clear(Rgb666::BLACK).unwrap();
        Timer::after_secs(2).await;
        display.clear(Rgb666::WHITE).unwrap();
        Timer::after_secs(2).await;
    }
}
