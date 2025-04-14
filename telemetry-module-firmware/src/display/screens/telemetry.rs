use crate::{
    display::{
        drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
        DrawType, DrawTypeDrawable,
    },
    network::telemetry_tx::{
        TELEMETRY_TX_BUFFER_SUBMISSIONS, TELEMETRY_TX_FAIL_BUFFER, TELEMETRY_TX_FAIL_NETWORK,
        TELEMETRY_TX_SUCCESS,
    },
    telemetry::machine::{TELEMETRY_RX_FAIL, TELEMETRY_RX_SUCCESS},
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
        let value_offset = 80;
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
                    let count = TELEMETRY_RX_SUCCESS.load(Ordering::Relaxed);
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
                    let count = TELEMETRY_RX_FAIL.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of metrics added to the transmit buffer
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Buf Ins",
            Some(
                {
                    let count = TELEMETRY_TX_BUFFER_SUBMISSIONS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of messages transmitted
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Msgs Tx",
            Some(
                {
                    let count = TELEMETRY_TX_SUCCESS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of buffer related transmit failures
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Buf",
            Some(
                {
                    let count = TELEMETRY_TX_FAIL_BUFFER.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of network related transmit failures
        let _cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Net",
            Some(
                {
                    let count = TELEMETRY_TX_FAIL_NETWORK.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", count)).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
