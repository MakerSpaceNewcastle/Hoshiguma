use crate::{
    display::{
        drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
        DrawType, DrawTypeDrawable,
    },
    telemetry::{TELEMETRY_EVENTS_RECEIVED, TELEMETRY_RECEIVE_FAILURES},
};
use core::{fmt::Write, sync::atomic::Ordering};
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
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Events Rx",
            Some(
                {
                    let count = TELEMETRY_EVENTS_RECEIVED.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of receive failures
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Rx Fail",
            Some(
                {
                    let count = TELEMETRY_RECEIVE_FAILURES.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of messages transmitted
        // TODO
        let cursor =
            Measurement::new(cursor, value_offset, "#Msgs Tx", None).draw(target, draw_type)?;

        // Number of transmit failures
        // TODO
        let _cursor =
            Measurement::new(cursor, value_offset, "#Tx Fail", None).draw(target, draw_type)?;

        Ok(())
    }
}
