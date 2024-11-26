use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, WebColors},
    text::{Alignment, Text},
    Drawable,
};

pub(crate) enum Severity {
    Normal,
    Warning,
    Critical,
}

pub(crate) struct Measurement<'a> {
    origin: Point,
    value_offset: Point,

    name: &'static str,

    value: Option<&'a str>,
    severity: Option<Severity>,
}

impl<'a> Measurement<'a> {
    fn height() -> i32 {
        12
    }

    pub(crate) fn new(
        origin: Point,
        value_offset: i32,
        name: &'static str,
        value: Option<&'a str>,
        severity: Option<Severity>,
    ) -> Self {
        Self {
            origin,
            value_offset: Point::new(value_offset, 0),
            name,
            value,
            severity,
        }
    }
}

impl Drawable for Measurement<'_> {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let name_style = MonoTextStyle::new(&FONT_6X10, super::LIGHT_TEXT_COLOUR);

        Text::with_alignment(self.name, self.origin, name_style, Alignment::Left).draw(target)?;

        let value_style = MonoTextStyle::new(
            &FONT_6X10,
            if self.value.is_some() {
                match self.severity {
                    Some(Severity::Normal) => Rgb565::CSS_GREEN,
                    Some(Severity::Warning) => Rgb565::CSS_YELLOW,
                    Some(Severity::Critical) => Rgb565::CSS_RED,
                    None => super::LIGHT_TEXT_COLOUR,
                }
            } else {
                Rgb565::CSS_MAGENTA
            },
        );

        let value = self.value.unwrap_or("<unknown>");

        Text::with_alignment(
            value,
            self.origin + self.value_offset,
            value_style,
            Alignment::Left,
        )
        .draw(target)?;

        Ok(self.origin + Point::new(0, Self::height()))
    }
}
