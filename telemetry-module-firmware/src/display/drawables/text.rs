use crate::display::{DrawType, DrawTypeDrawable, LIGHT_TEXT_COLOUR};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    text::{Alignment, Text},
    Drawable,
};

/// Generic simple text.
pub(crate) struct GenericText<'a> {
    origin: Point,

    text: &'a str,
    pub(crate) style: MonoTextStyle<'static, Rgb565>,
}

impl<'a> GenericText<'a> {
    pub(crate) fn height() -> i32 {
        12
    }

    pub(crate) fn new(origin: Point, text: &'a str) -> Self {
        Self {
            origin,
            text,
            style: MonoTextStyle::new(&FONT_6X10, LIGHT_TEXT_COLOUR),
        }
    }
}

impl DrawTypeDrawable for GenericText<'_> {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        if *draw_type == DrawType::Full {
            Text::with_alignment(self.text, self.origin, self.style, Alignment::Left)
                .draw(target)?;
        }

        Ok(self.origin + Point::new(0, Self::height()))
    }
}
