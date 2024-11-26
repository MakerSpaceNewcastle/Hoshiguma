use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    text::{renderer::CharacterStyle, Alignment, DecorationColor, Text},
    Drawable,
};

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

impl Drawable for Subtitle {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut style = MonoTextStyle::new(&FONT_6X10, super::LIGHT_TEXT_COLOUR);
        style.set_underline_color(DecorationColor::TextColor);

        Text::with_alignment(self.text, self.origin, style, Alignment::Left).draw(target)?;

        Ok(self.origin + Point::new(0, Self::height()))
    }
}
