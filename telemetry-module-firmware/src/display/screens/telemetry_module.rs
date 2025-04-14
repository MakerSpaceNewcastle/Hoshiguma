use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embassy_time::Instant;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct TelemetryModule {}

impl DrawTypeDrawable for TelemetryModule {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 25;
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // Git revision of the telemetry module
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Rev",
            Some(git_version::git_version!()),
        )
        .draw(target, draw_type)?;

        // Uptime of the telemetry module
        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{}s", Instant::now().as_secs()))
            .unwrap();
        let cursor =
            Measurement::new(cursor, value_offset, "Up", Some(&s)).draw(target, draw_type)?;

        // Boot reason of the telemetry module
        let boot_reason = embassy_rp::pac::WATCHDOG.reason().read();
        let _cursor = Measurement::new(
            cursor,
            value_offset,
            "Bt.",
            Some(if boot_reason.force() {
                "Forced Reset"
            } else if boot_reason.timer() {
                "Watchdog Timeout"
            } else {
                "Normal"
            }),
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
