use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, WebColors},
    primitives::{Line, PrimitiveStyleBuilder, StrokeAlignment},
    text::{Alignment, Text},
    Drawable,
};

#[derive(Default)]
pub(crate) struct BootScreen {}

impl Drawable for BootScreen {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_BLACK);

        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::CSS_YELLOW)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        let display_box = target.bounding_box();

        // Fill the display with black pixels
        target.clear(Rgb565::CSS_HOT_PINK)?;

        // Draw a one pixel border around the display
        display_box.into_styled(line_style).draw(target)?;

        // Draw a line through the display area
        Line::new(
            display_box
                .bottom_right()
                .expect("the display bounding box should be of size > 1x1"),
            display_box.top_left,
        )
        .into_styled(line_style)
        .draw(target)?;

        // Show some text
        Text::with_alignment(
            "Hoshiguma\nTelemetry Adapter",
            display_box.center(),
            text_style,
            Alignment::Center,
        )
        .draw(target)?;

        // Show the firmware version
        Text::with_alignment(
            git_version::git_version!(),
            display_box.center() + Point::new(0, 50),
            text_style,
            Alignment::Center,
        )
        .draw(target)?;

        Ok(())
    }
}
