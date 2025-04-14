use core::fmt::Write;

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

        // Time
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Time",
            crate::network::time::wall_time()
                .map(|time| {
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", time.as_secs())).unwrap();
                    s
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        // Seconds since last time sync
        let _cursor = Measurement::new(
            cursor,
            value_offset,
            "Age",
            Some(
                {
                    let age = crate::network::time::time_sync_age().as_secs();
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}s", age)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
