use crate::display::{
    drawables::title_bar::HEIGHT as TITLE_BAR_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, Size, WebColors},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

pub(crate) const REGION: Rectangle = Rectangle::new(
    Point::new(0, TITLE_BAR_HEIGHT as i32),
    Size::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32 - TITLE_BAR_HEIGHT),
);
const BACKGROUND_COLOUR: Rgb565 = Rgb565::CSS_BLACK;

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
            .fill_color(BACKGROUND_COLOUR)
            .build();

        REGION.into_styled(background_style).draw(target)
    }
}
