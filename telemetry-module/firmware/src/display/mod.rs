mod drawables;
mod screens;
pub(super) mod state;

use crate::ui_button::{UiEvent, UI_INPUTS};
use core::cell::RefCell;
use defmt::info;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_futures::select::{select3, Either3};
use embassy_rp::{
    gpio::{Level, Output},
    spi::Config,
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::Timer;
use embedded_graphics::Drawable;
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

    let ui_event_rx = UI_INPUTS.receiver();

    loop {
        match select3(
            ui_event_rx.receive(),
            STATE_CHANGED.wait(),
            embassy_time::Timer::after_secs(5),
        )
        .await
        {
            Either3::First(msg) => match msg {
                UiEvent::ButtonPushed => {
                    screen_selector.select_next();
                }
            },
            Either3::Second(new_state) => {
                state = new_state;
            }
            Either3::Third(_) => {
                // Nothing special to do here, just redraw the screen
            }
        };

        // TODO: only redraw full screen when it changes (otherwise just redraw values)
        info!("Display draw");
        draw_drawable!(
            &mut display,
            drawables::title_bar::TitleBar::new(&screen_selector)
        );
        draw_drawable!(
            &mut display,
            drawables::info_pane_background::InfoPaneBackground::default()
        );
        screen_selector.draw(&mut display, &state);
    }
}
