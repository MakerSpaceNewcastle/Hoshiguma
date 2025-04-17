use super::info_background::INFO_PANE_BACKGROUND_COLOUR;
use crate::display::{DrawType, DrawTypeDrawable, LIGHT_TEXT_COLOUR, SCREEN_WIDTH};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, Size, WebColors},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
    Drawable,
};

pub(crate) const UNKNOWN_TEXT: &str = "<unknown>";
pub(crate) const UNKNOWN_COLOUR: Rgb565 = Rgb565::CSS_MAGENTA;

pub(crate) enum Severity {
    Normal,
    Warning,
    Critical,
}

/// Shows the name and value of a measurement/state.
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

const NAME_TEXT_STYLE: MonoTextStyle<'_, Rgb565> =
    MonoTextStyle::new(&FONT_6X10, LIGHT_TEXT_COLOUR);

impl DrawTypeDrawable for Measurement<'_> {
    type Color = Rgb565;
    type Output = Point;

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_position = self.origin + self.value_offset;

        match draw_type {
            DrawType::Full => {
                // Draw the measurement name on a full redraw only
                Text::with_alignment(self.name, self.origin, NAME_TEXT_STYLE, Alignment::Left)
                    .draw(target)?;
            }
            DrawType::ValuesOnly => {
                // Blank the area where the value text will be drawn
                // (only required when doing a values only draw)
                let width = SCREEN_WIDTH as i32 - value_position.x;
                let region = Rectangle::new(
                    value_position - Point::new(0, Self::height() - 4),
                    Size::new(width as u32, Self::height() as u32),
                );
                let background_style = PrimitiveStyleBuilder::new()
                    .fill_color(INFO_PANE_BACKGROUND_COLOUR)
                    .build();
                region.into_styled(background_style).draw(target)?;
            }
        }

        let value_style = MonoTextStyle::new(
            &FONT_6X10,
            if self.value.is_some() {
                match self.severity {
                    Some(Severity::Normal) => Rgb565::CSS_GREEN,
                    Some(Severity::Warning) => Rgb565::CSS_YELLOW,
                    Some(Severity::Critical) => Rgb565::CSS_RED,
                    None => LIGHT_TEXT_COLOUR,
                }
            } else {
                UNKNOWN_COLOUR
            },
        );

        let value = self.value.unwrap_or(UNKNOWN_TEXT);

        // Always draw the value text
        Text::with_alignment(value, value_position, value_style, Alignment::Left).draw(target)?;

        Ok(self.origin + Point::new(0, Self::height()))
    }
}
