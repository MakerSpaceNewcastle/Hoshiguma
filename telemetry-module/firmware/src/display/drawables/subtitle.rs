use crate::display::{DrawType, DrawTypeDrawable};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    text::{renderer::CharacterStyle, Alignment, DecorationColor, Text},
    Drawable,
};

/// Underlined text that can be used to group sets of related values on screen.
pub(crate) struct Subtitle {
    origin: Point,

    text: &'static str,
}

impl Subtitle {
    fn height() -> i32 {
        12
    }

    pub(crate) fn new(origin: Point, text: &'static str) -> Self {
        Self { origin, text }
    }
}

impl DrawTypeDrawable for Subtitle {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut style = MonoTextStyle::new(&FONT_6X10, super::LIGHT_TEXT_COLOUR);
        style.set_underline_color(DecorationColor::TextColor);

        if *draw_type == DrawType::Full {
            Text::with_alignment(self.text, self.origin, style, Alignment::Left).draw(target)?;
        }

        Ok(self.origin + Point::new(0, Self::height()))
    }
}
