use super::screen::TITLE_BAR_HEIGHT;
use crate::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, Size, WebColors},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

pub(crate) const INFO_PANE_REGION: Rectangle = Rectangle::new(
    Point::new(0, TITLE_BAR_HEIGHT as i32),
    Size::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32 - TITLE_BAR_HEIGHT),
);
pub(crate) const INFO_PANE_BACKGROUND_COLOUR: Rgb565 = Rgb565::CSS_BLACK;

/// Background of the information pane
#[derive(Default)]
pub(crate) struct InfoPaneBackground {}

impl Drawable for InfoPaneBackground {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let background_style = PrimitiveStyleBuilder::new()
            .fill_color(INFO_PANE_BACKGROUND_COLOUR)
            .build();

        INFO_PANE_REGION
            .into_styled(background_style)
            .draw(target)?;

        Ok(())
    }
}
