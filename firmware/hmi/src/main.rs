#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    pixelcolor::Rgb666,
    prelude::{Point, Primitive, RgbColor},
    primitives::{Line, PrimitiveStyle},
};
use panic_probe as _;
use peek_o_display_bsp::{
    PeekODisplay,
    display::Rotation,
    embassy_rp::gpio::{Level, Output},
    touch::Calibration,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = PeekODisplay::default();
    let p = board.peripherals();

    let spi = board.board_spi();

    let display_rotation = Rotation::Deg0;

    let (mut display, _backlight) = board.display(spi, display_rotation);
    let (mut touch, touch_irq) = board.touch(spi, display_rotation, Calibration::default());

    display.clear(Rgb666::BLACK).unwrap();

    let mut ticker = Ticker::every(Duration::from_hz(100));

    let mut last = (0i32, 0i32);

    let mut led = Output::new(p.PIN_19, Level::Low);

    touch.read();

    loop {
        ticker.next().await;

        let irq_level = touch_irq.get_level();
        led.set_level(irq_level);

        if irq_level == Level::Low
            && let Some(point) = touch.read()
        {
            info!("touch at : {},{}", point.0, point.1);

            draw_cursor(&mut display, last, Rgb666::BLACK, Rgb666::BLACK);
            draw_cursor(&mut display, point, Rgb666::RED, Rgb666::GREEN);

            last = point;
        }
    }
}

fn draw_cursor<D: DrawTarget<Color = Rgb666>>(
    display: &mut D,
    point: (i32, i32),
    x_color: Rgb666,
    y_color: Rgb666,
) where
    <D as DrawTarget>::Error: core::fmt::Debug,
{
    let bbox = display.bounding_box();

    // Draw a vertical line at x
    Line::new(
        Point::new(point.0, bbox.bottom_right().unwrap().y),
        Point::new(point.0, bbox.top_left.y),
    )
    .into_styled(PrimitiveStyle::with_stroke(x_color, 1))
    .draw(display)
    .unwrap();

    // Draw a horizontal line a y
    Line::new(
        Point::new(bbox.bottom_right().unwrap().x, point.1),
        Point::new(bbox.top_left.x, point.1),
    )
    .into_styled(PrimitiveStyle::with_stroke(y_color, 1))
    .draw(display)
    .unwrap();
}
