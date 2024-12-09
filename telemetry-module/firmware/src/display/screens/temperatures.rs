use crate::display::{
    drawables::{measurement::Measurement, screen::INFO_PANE_REGION},
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Temperatures<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Temperatures<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Temperatures<'_> {
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

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Board",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.onboard.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Elec Bay",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.electronics_bay_top.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Laser Bay",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.laser_chamber.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Ambient",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.ambient.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Cool Flow",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.coolant_flow.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Cool Ret",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.coolant_return.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Res Top",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.coolant_resevoir_top.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Res Bottom",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.coolant_resevoir_bottom.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        Measurement::new(
            cursor,
            value_offset,
            "Cool Pump",
            self.state
                .temperatures
                .as_ref()
                .and_then(|temperatures| {
                    temperatures.coolant_pump.ok().map(|t| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{} C", t)).unwrap();
                        s
                    })
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
