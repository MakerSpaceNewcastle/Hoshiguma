use crate::display::{
    drawables::{
        info_background::INFO_PANE_REGION,
        measurement::{Measurement, Severity},
    },
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};
use hoshiguma_protocol::peripheral_controller::types::{
    AirAssistDemand, ChassisIntrusion, CoolantResevoirLevel, FumeExtractionMode, MachinePower,
    MachineRun,
};

pub(super) struct Inputs<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Inputs<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Inputs<'_> {
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

        // Machine power detection
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Mach Power",
            self.state.machine_power.as_ref().map(|v| match v {
                MachinePower::On => "On",
                MachinePower::Off => "Off",
            }),
            self.state.machine_power.as_ref().map(|v| match v {
                MachinePower::On => Severity::Normal,
                MachinePower::Off => Severity::Critical,
            }),
        )
        .draw(target, draw_type)?;

        // Machine running
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Mach Run",
            self.state.machine_run_status.as_ref().map(|v| match v {
                MachineRun::Idle => "Idle",
                MachineRun::Running => "Running",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Air assist demand
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Air Demand",
            self.state.air_assist_demand.as_ref().map(|v| match v {
                AirAssistDemand::Idle => "Idle",
                AirAssistDemand::Demand => "Demand",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Fume extraction mode switch
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Extr Mode",
            self.state.fume_extraction_mode.as_ref().map(|v| match v {
                FumeExtractionMode::Automatic => "Auto",
                FumeExtractionMode::OverrideRun => "Manual",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Chassis intrusion
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Chassis",
            self.state.chassis_intrusion.as_ref().map(|v| match v {
                ChassisIntrusion::Normal => "OK",
                ChassisIntrusion::Intruded => "Intrusion",
            }),
            None,
        )
        .draw(target, draw_type)?;

        // Coolant resevoir level
        Measurement::new(
            cursor,
            value_offset,
            "Cool Level",
            match self.state.coolant_resevoir_level.as_ref() {
                Some(Ok(CoolantResevoirLevel::Full)) => Some("Full"),
                Some(Ok(CoolantResevoirLevel::Low)) => Some("Low"),
                Some(Ok(CoolantResevoirLevel::Empty)) => Some("Empty"),
                _ => None,
            },
            match self.state.coolant_resevoir_level.as_ref() {
                Some(Ok(CoolantResevoirLevel::Full)) => Some(Severity::Normal),
                Some(Ok(CoolantResevoirLevel::Low)) => Some(Severity::Warning),
                Some(Ok(CoolantResevoirLevel::Empty)) => Some(Severity::Critical),
                _ => None,
            },
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
