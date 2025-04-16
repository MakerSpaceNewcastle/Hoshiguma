mod drawables;

use core::cell::RefCell;
use defmt::warn;
use drawables::boot_screen::BootScreen;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::{
    gpio::{Level, Output},
    spi::{Config as SpiConfig, Spi},
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::Timer;
use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors, Drawable};
use mipidsi::{
    interface::SpiInterface,
    models::ST7735s,
    options::{ColorOrder, Orientation, Rotation},
};

const SCREEN_WIDTH: u16 = 128;
const SCREEN_HEIGHT: u16 = 128;

const LIGHT_TEXT_COLOUR: Rgb565 = Rgb565::CSS_MOCCASIN;

#[embassy_executor::task]
pub(super) async fn task(r: crate::DisplayResources) {
    let mut config = SpiConfig::default();
    config.frequency = 64_000_000;

    let spi = Spi::new_blocking(r.spi, r.clk, r.mosi, r.miso, config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(r.cs, Level::High), config);

    let dc = Output::new(r.dc, Level::Low);
    let rst = Output::new(r.rst, Level::Low);
    let _led = Output::new(r.led, Level::Low);

    let mut buffer = [0_u8; 512];
    let interface = SpiInterface::new(display_spi, dc, &mut buffer);

    let mut display = mipidsi::Builder::new(ST7735s, interface)
        .display_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .display_offset(2, 31)
        .orientation(Orientation::default().rotate(Rotation::Deg270))
        .color_order(ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut embassy_time::Delay)
        .unwrap();

    // Show the boot splash screen
    if BootScreen::default().draw(&mut display).is_err() {
        warn!("Failed to draw boot screen");
    }
    Timer::after_secs(2).await;

    // TODO
}
