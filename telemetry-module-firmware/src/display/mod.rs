mod drawables;
mod screens;

use self::screens::ScreenSelector;
use crate::ui_button::{UiEvent, UI_INPUTS};
use core::cell::RefCell;
use defmt::{debug, warn, Format};
use drawables::{boot_screen::BootScreen, screen::Screen};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio::{Level, Output},
    spi::{Config as SpiConfig, Spi},
};
use embassy_sync::{
    blocking_mutex::{raw::NoopRawMutex, Mutex},
    pubsub::WaitResult,
};
use embassy_time::Timer;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, WebColors},
    Drawable,
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7735s,
    options::{ColorOrder, Orientation, Rotation},
};

const SCREEN_WIDTH: u16 = 128;
const SCREEN_HEIGHT: u16 = 128;

const LIGHT_TEXT_COLOUR: Rgb565 = Rgb565::CSS_MOCCASIN;

#[derive(PartialEq, Eq, Format)]
enum DrawType {
    Full,
    ValuesOnly,
}

trait DrawTypeDrawable {
    type Color;
    type Output;

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::DisplayResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("display").await;

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

    let mut screen_selector = ScreenSelector::default();

    let mut ui_event_rx = UI_INPUTS.subscriber().unwrap();

    // Initial full display draw
    draw(&mut display, DrawType::Full, &screen_selector).await;

    loop {
        let draw_type = match select(ui_event_rx.next_message(), Timer::after_secs(1)).await {
            Either::First(msg) => match msg {
                WaitResult::Message(UiEvent::ButtonPushed) => {
                    screen_selector.select_next();
                    Some(DrawType::Full)
                }
                _ => None,
            },
            Either::Second(_) => {
                // Nothing special to do here, just redraw the screen
                Some(DrawType::ValuesOnly)
            }
        };

        if let Some(draw_type) = draw_type {
            draw(&mut display, draw_type, &screen_selector).await;
        }
    }
}

async fn draw<D>(display: &mut D, draw_type: DrawType, screen_selector: &ScreenSelector)
where
    D: DrawTarget<Color = Rgb565>,
{
    debug!("Display draw ({})", draw_type);

    if draw_type == DrawType::Full && Screen::new(screen_selector).draw(display).is_err() {
        warn!("Failed to draw screen title");
    }

    if screen_selector.draw(display, &draw_type).is_err() {
        warn!("Failed to draw screen");
    }
}
