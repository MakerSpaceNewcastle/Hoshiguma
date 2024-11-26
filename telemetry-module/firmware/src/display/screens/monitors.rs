use crate::display::drawables::info_pane_background::REGION;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, WebColors},
    text::{Alignment, Text},
    Drawable,
};

pub(super) struct Monitors {}

impl Drawable for Monitors {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::with_alignment(
            "TODO\nMonitors",
            REGION.center(),
            MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_WHITE),
            Alignment::Center,
        )
        .draw(target)?;

        Ok(())
    }
}
