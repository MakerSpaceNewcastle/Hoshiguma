use crate::display::{
    drawables::{measurement::Measurement, screen::INFO_PANE_REGION, subtitle::Subtitle},
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embassy_time::Instant;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Device<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Device<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Device<'_> {
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

        let cursor = Subtitle::new(cursor, "Telemetry Module").draw(target, draw_type)?;

        // Git revision of the telemetry module
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Rev",
            Some(git_version::git_version!()),
            None,
        )
        .draw(target, draw_type)?;

        // Uptime of the telemetry module
        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{}s", Instant::now().as_secs()))
            .unwrap();
        let cursor =
            Measurement::new(cursor, value_offset, "Up", Some(&s), None).draw(target, draw_type)?;

        let cursor = cursor + Point::new(0, 5);

        let cursor = Subtitle::new(cursor, "Controller").draw(target, draw_type)?;

        // Git revision of the controller
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Rev",
            self.state.controller_git_rev.as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        // Uptime of the controller
        Measurement::new(
            cursor,
            value_offset,
            "Up",
            self.state
                .controller_uptime
                .map(|uptime| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}s", uptime / 1000)).unwrap();
                    s
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
