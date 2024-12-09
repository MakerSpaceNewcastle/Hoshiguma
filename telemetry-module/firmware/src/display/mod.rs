mod drawables;
mod screens;
pub(super) mod state;

use crate::ui_button::{UiEvent, UI_INPUTS};
use core::cell::RefCell;
use defmt::{debug, Format};
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_futures::select::{select3, Either3};
use embassy_rp::{
    gpio::{Level, Output},
    spi::Config,
};
use embassy_sync::{
    blocking_mutex::{raw::NoopRawMutex, Mutex},
    pubsub::WaitResult,
};
use embassy_time::Timer;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget, Drawable};
use mipidsi::{
    models::ST7735s,
    options::{ColorOrder, Orientation, Rotation},
};
use screens::ScreenSelector;
use state::{DisplayDataState, STATE_CHANGED};

macro_rules! draw_drawable {
    ($display: expr, $drawable: expr) => {{
        if let Err(_) = $drawable.draw($display) {
            defmt::error!("Failed to draw drawable");
        }
    }};
}

const SCREEN_WIDTH: u16 = 128;
const SCREEN_HEIGHT: u16 = 128;

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
    let mut config = Config::default();
    config.frequency = 16_000_000;

    let spi = embassy_rp::spi::Spi::new_blocking(r.spi, r.clk, r.mosi, r.miso, config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, Output::new(r.cs, Level::High), config);

    let dc = Output::new(r.dc, Level::Low);
    let rst = Output::new(r.rst, Level::Low);
    let _led = Output::new(r.led, Level::Low);

    let interface = SPIInterface::new(display_spi, dc);

    let mut display = mipidsi::Builder::new(ST7735s, interface)
        .display_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .display_offset(2, 31)
        .orientation(Orientation::default().rotate(Rotation::Deg270))
        .color_order(ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut embassy_time::Delay)
        .unwrap();

    // Show the boot splash screen
    draw_drawable!(&mut display, drawables::boot_screen::BootScreen::default());
    Timer::after_secs(2).await;

    let mut screen_selector = ScreenSelector::default();
    let mut state = DisplayDataState::default();

    let mut ui_event_rx = UI_INPUTS.subscriber().unwrap();

    // Initial full display draw
    draw(&mut display, DrawType::Full, &screen_selector, &state).await;

    loop {
        let draw_type = match select3(
            ui_event_rx.next_message(),
            STATE_CHANGED.wait(),
            embassy_time::Timer::after_secs(5),
        )
        .await
        {
            Either3::First(msg) => match msg {
                WaitResult::Message(UiEvent::ButtonPushed) => {
                    screen_selector.select_next();
                    Some(DrawType::Full)
                }
                _ => None,
            },
            Either3::Second(new_state) => {
                state = new_state;
                Some(DrawType::ValuesOnly)
            }
            Either3::Third(_) => {
                // Nothing special to do here, just redraw the screen
                // TODO: is this actually needed?
                Some(DrawType::ValuesOnly)
            }
        };

        if let Some(draw_type) = draw_type {
            draw(&mut display, draw_type, &screen_selector, &state).await;
        }
    }
}

async fn draw<D>(
    display: &mut D,
    draw_type: DrawType,
    screen_selector: &ScreenSelector,
    state: &DisplayDataState,
) where
    D: DrawTarget<Color = Rgb565>,
{
    debug!("Display draw ({})", draw_type);
    if draw_type == DrawType::Full {
        draw_drawable!(display, drawables::screen::Screen::new(screen_selector));
    }
    screen_selector.draw(display, &draw_type, state);
}
