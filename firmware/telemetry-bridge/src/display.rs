use crate::DisplayResources;
use embassy_rp::{
    bind_interrupts,
    i2c::{I2c, InterruptHandler},
    peripherals::I2C1,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
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
use heapless::String;
use ssd1306::{
    I2CDisplayInterface, Ssd1306, mode::DisplayConfig, prelude::DisplayRotation,
    size::DisplaySize128x64,
};

pub(crate) type DisplayText = String<256>;

pub(crate) static DISPLAY_TEXT: Watch<CriticalSectionRawMutex, DisplayText, 1> = Watch::new();

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

    let mut text_rx = DISPLAY_TEXT.receiver().unwrap();

    loop {
        let text = text_rx.changed().await;

        let text_box = TextBox::with_textbox_style(&text, bounds, character_style, textbox_style);

        display.clear_buffer();
        text_box.draw(&mut display).unwrap();
        display.flush().unwrap();

        Timer::after_secs(1).await;
    }
}
