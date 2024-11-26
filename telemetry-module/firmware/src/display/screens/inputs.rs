use crate::display::{
    drawables::{
        info_pane_background::REGION,
        measurement::{Measurement, Severity},
    },
    state::DisplayDataState,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    Drawable,
};
use hoshiguma_telemetry_protocol::payload::observation::{
    AirAssistDemand, ChassisIntrusion, CoolantResevoirLevel, FumeExtractionMode, MachinePower,
    MachineRunStatus,
};

pub(super) struct Inputs<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Inputs<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl Drawable for Inputs<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 65;
        let cursor = Point::new(REGION.top_left.x + 2, REGION.top_left.y + 11);

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
        .draw(target)?;

        // Machine running
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Mach Run",
            self.state.machine_run_status.as_ref().map(|v| match v {
                MachineRunStatus::Idle => "Idle",
                MachineRunStatus::Running => "Running",
            }),
            None,
        )
        .draw(target)?;

        // Air assist demand
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Air Demand",
            self.state.air_assist_demand.as_ref().map(|v| match v {
                AirAssistDemand::Idle => "Demand",
                AirAssistDemand::Demand => "Idle",
            }),
            None,
        )
        .draw(target)?;

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
        .draw(target)?;

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
        .draw(target)?;

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
        .draw(target)?;

        Ok(())
    }
}
