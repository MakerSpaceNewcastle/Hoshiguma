use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
    DrawType, DrawTypeDrawable,
};
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
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // NTP server address
        // TODO
        let cursor = Measurement::new(cursor, value_offset, "NTP", None).draw(target, draw_type)?;

        // Seconds since last time sync
        // TODO
        let cursor = Measurement::new(cursor, value_offset, "Age", None).draw(target, draw_type)?;

        // Time
        // TODO
        let _cursor =
            Measurement::new(cursor, value_offset, "Time", None).draw(target, draw_type)?;

        Ok(())
    }
}
