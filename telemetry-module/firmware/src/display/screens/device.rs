use crate::display::{
    drawables::{info_pane_background::REGION, measurement::Measurement, subtitle::Subtitle},
    state::DisplayDataState,
};
use core::fmt::Write;
use embassy_time::Instant;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    Drawable,
};

pub(super) struct Device<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Device<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl Drawable for Device<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 25;
        let cursor = Point::new(REGION.top_left.x + 2, REGION.top_left.y + 11);

        let cursor = Subtitle::new(cursor, "Telemetry Module").draw(target)?;

        // Git revision of the telemetry module
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Rev",
            Some(git_version::git_version!()),
            None,
        )
        .draw(target)?;

        // Uptime of the telemetry module
        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{}s", Instant::now().as_secs()))
            .unwrap();
        let cursor = Measurement::new(cursor, value_offset, "Up", Some(&s), None).draw(target)?;

        let cursor = cursor + Point::new(0, 5);

        let cursor = Subtitle::new(cursor, "Controller").draw(target)?;

        // Git revision of the controller
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Rev",
            self.state.controller_git_rev.as_deref(),
            None,
        )
        .draw(target)?;

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
        .draw(target)?;

        Ok(())
    }
}
