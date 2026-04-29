use crate::DisplayResources;
use chrono::{Datelike, Timelike};
use core::fmt::Write;
use embassy_rp::{
    bind_interrupts,
    i2c::{I2c, InterruptHandler},
    peripherals::I2C1,
};
use embassy_time::Timer;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::{Point, Size},
    primitives::Rectangle,
};
use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};
use ssd1306::{
    I2CDisplayInterface, Ssd1306, mode::DisplayConfig, prelude::DisplayRotation,
    size::DisplaySize128x64,
};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: DisplayResources) -> ! {
    let i2c = I2c::new_async(r.i2c, r.scl, r.sda, Irqs, Default::default());

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(2)
        .build();

    let bounds = Rectangle::new(Point::zero(), Size::new(128, 0));

    loop {
        let mut text = heapless::String::<128>::new();

        text.write_str("Rev: ").unwrap();
        text.write_str(git_version::git_version!()).unwrap();
        text.write_char('\n').unwrap();

        let time = crate::wall_time::now();

        text.write_str("Date: ").unwrap();
        match time {
            Some(time) => text.write_fmt(format_args!(
                "{}-{:02}-{:02}\n",
                time.year(),
                time.month(),
                time.day()
            )),
            None => text.write_str("unknown\n"),
        }
        .unwrap();

        text.write_str("Time: ").unwrap();
        match time {
            Some(time) => text.write_fmt(format_args!(
                "{:02}:{:02}:{:02}\n",
                time.hour(),
                time.minute(),
                time.second()
            )),
            None => text.write_str("unknown\n"),
        }
        .unwrap();

        let text_box = TextBox::with_textbox_style(&text, bounds, character_style, textbox_style);

        display.clear_buffer();
        text_box.draw(&mut display).unwrap();
        display.flush().unwrap();

        Timer::after_secs(1).await;
    }
}
