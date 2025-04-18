use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Telemetry {}

impl DrawTypeDrawable for Telemetry {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 65;
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // Number of events received from peripheral controller
        // TODO
        let cursor = Measurement::new(cursor, value_offset, "#events rx", Some("0"))
            .draw(target, draw_type)?;

        // Number of receive failures
        // TODO
        let cursor =
            Measurement::new(cursor, value_offset, "#rx fail", None).draw(target, draw_type)?;

        // Number of messages transmitted
        // TODO
        let cursor =
            Measurement::new(cursor, value_offset, "#msgs tx", None).draw(target, draw_type)?;

        // Number of transmit failures
        // TODO
        let _cursor =
            Measurement::new(cursor, value_offset, "#tx fail", None).draw(target, draw_type)?;

        Ok(())
    }
}
