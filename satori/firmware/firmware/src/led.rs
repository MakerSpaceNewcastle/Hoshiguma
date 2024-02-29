use log::error;
use smart_leds::{SmartLedsWrite, RGB8};
use std::sync::{Arc, Mutex};
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

pub(crate) const BLACK: RGB8 = RGB8::new(0, 0, 0);
pub(crate) const RED: RGB8 = RGB8::new(8, 0, 0);
pub(crate) const GREEN: RGB8 = RGB8::new(0, 8, 0);
pub(crate) const BLUE: RGB8 = RGB8::new(0, 0, 8);

pub(crate) struct Led {
    leds: Arc<Mutex<Ws2812Esp32Rmt>>,
}

impl Led {
    pub(crate) fn new() -> Self {
        let leds = Ws2812Esp32Rmt::new(0, 8).expect("WS2812 driver should be configured");

        Self {
            leds: Arc::new(Mutex::new(leds)),
        }
    }

    pub(crate) fn set(&self, colour: RGB8) {
        let mut leds = self.leds.lock().unwrap();
        if let Err(e) = leds.write([colour].into_iter()) {
            error!("Failed to write to LED (error: {:?})", e);
        }
    }
}
