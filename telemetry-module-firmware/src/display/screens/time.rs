use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Time {}

impl DrawTypeDrawable for Time {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 35;
        let mut cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        cursor = Measurement::new(cursor, value_offset, "Gtwy", None).draw(target, draw_type)?;

        Ok(())
    }
}
