use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};
use hoshiguma_protocol::payload::control::{
    AirAssistPump, FumeExtractionFan, LaserEnable, MachineEnable,
};

pub(super) struct Outputs<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Outputs<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Outputs<'_> {
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

        // Status lamp
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Status L",
            self.state
                .status_lamp
                .as_ref()
                .map(|sl| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!(
                        "{} {} {}",
                        match sl.red {
                            true => "R",
                            false => " ",
                        },
                        match sl.amber {
                            true => "A",
                            false => " ",
                        },
                        match sl.green {
                            true => "G",
                            false => " ",
                        },
                    ))
                    .unwrap();
                    s
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        // Machine enable
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Mach En",
            self.state.machine_enable.as_ref().map(|v| match v {
                MachineEnable::Inhibited => "Inhibited",
                MachineEnable::Enabled => "Enabled",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Laser enable
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Laser En",
            self.state.laser_enable.as_ref().map(|v| match v {
                LaserEnable::Inhibited => "Inhibited",
                LaserEnable::Enabled => "Enabled",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Air assist pump
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Air Pump",
            self.state.air_assist_pump.as_ref().map(|v| match v {
                AirAssistPump::Idle => "Idle",
                AirAssistPump::Demand => "Run",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Fume extractor fan
        Measurement::new(
            cursor,
            value_offset,
            "Fume Extr",
            self.state.fume_extraction_fan.as_ref().map(|v| match v {
                FumeExtractionFan::Idle => "Idle",
                FumeExtractionFan::Demand => "Run",
            }),
            None,
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
