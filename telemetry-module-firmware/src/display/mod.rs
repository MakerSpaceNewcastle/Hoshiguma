mod drawables;

use core::{cell::RefCell, convert::Infallible};
use defmt::{warn, Format};
use drawables::boot_screen::BootScreen;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::{
    gpio::{Level, Output},
    pwm::{Pwm, SetDutyCycle},
    spi::{Config as SpiConfig, Spi},
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, WebColors},
    Drawable,
};
use embedded_hal::digital::{ErrorType, OutputPin};
use mipidsi::{interface::SpiInterface, models::ST7789, options::ColorInversion, Builder};

const SCREEN_WIDTH: u16 = 240;
const SCREEN_HEIGHT: u16 = 240;

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
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    let spi = Spi::new_blocking_txonly(r.spi, r.clk_pin, r.mosi_pin, config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(&spi_bus, NoCs, config);

    let dc = Output::new(r.dc_pin, Level::Low);
    let rst = Output::new(r.reset_pin, Level::Low);

    let mut backlight = Pwm::new_output_a(
        r.backlight_pwm,
        r.backlight_pin,
        embassy_rp::pwm::Config::default(),
    );
    let _ = backlight.set_duty_cycle_fully_on();

    let mut buffer = [0_u8; 512];
    let interface = SpiInterface::new(display_spi, dc, &mut buffer);

    let mut display = Builder::new(ST7789, interface)
        .display_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut Delay)
        .unwrap();

    // Show the boot splash screen
    if BootScreen::default().draw(&mut display).is_err() {
        warn!("Failed to draw boot screen");
    }
    Timer::after_secs(2).await;

    let _ = display.clear(Rgb565::CSS_BLACK);

    let page = self::drawables::diagnostics::Diagnostics {};
    let _ = page.draw(&mut display, &DrawType::Full);

    loop {
        let _ = page.draw(&mut display, &DrawType::ValuesOnly);
        Timer::after_secs(1).await;
    }
}

struct NoCs;

impl OutputPin for NoCs {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ErrorType for NoCs {
    type Error = Infallible;
}
