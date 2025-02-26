use super::text::GenericText;
use crate::display::{DrawType, DrawTypeDrawable};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    text::{renderer::CharacterStyle, DecorationColor},
};

/// Underlined text that can be used to group sets of related values on screen.
pub(crate) struct Subtitle<'a> {
    text: GenericText<'a>,
}

impl<'a> Subtitle<'a> {
    pub(crate) fn new(origin: Point, text: &'a str) -> Self {
        let mut text = GenericText::new(origin, text);
        text.style.set_underline_color(DecorationColor::TextColor);
        Self { text }
    }
}

impl DrawTypeDrawable for Subtitle<'_> {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.text.draw(target, draw_type)
    }
}
