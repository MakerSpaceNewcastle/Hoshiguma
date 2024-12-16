use crate::display::{
    drawables::{
        info_background::INFO_PANE_REGION,
        measurement::{Measurement, Severity},
    },
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};
use hoshiguma_telemetry_protocol::payload::{
    observation::MachineRunStatus, process::MachineOperationLockout,
};

pub(super) struct Summary<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Summary<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Summary<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 60;
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // Operation inhibit state
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Operation",
            self.state.lockout.as_ref().map(|v| match v {
                MachineOperationLockout::Permitted => "Permit",
                MachineOperationLockout::PermittedUntilIdle => "Deny Soon",
                MachineOperationLockout::Denied => "Deny",
            }),
            self.state.lockout.as_ref().map(|v| match v {
                MachineOperationLockout::Permitted => Severity::Normal,
                MachineOperationLockout::PermittedUntilIdle => Severity::Warning,
                MachineOperationLockout::Denied => Severity::Critical,
            }),
        )
        .draw(target, draw_type)?;

        // Number of active alarms
        let num_alarms = self.state.alarms.as_ref().map(|alarms| alarms.alarms.len());
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Alarms",
            num_alarms
                .as_ref()
                .map(|num_alarms| {
                    let mut s = heapless::String::<8>::new();
                    s.write_fmt(format_args!("{}", num_alarms)).unwrap();
                    s
                })
                .as_deref(),
            num_alarms.as_ref().map(|_| Severity::Warning),
        )
        .draw(target, draw_type)?;

        let cursor = cursor + Point::new(0, 5);

        // Machine running state
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Run",
            self.state.machine_run_status.as_ref().map(|v| match v {
                MachineRunStatus::Idle => "Idle",
                MachineRunStatus::Running => "Running",
            }),
            self.state.machine_run_status.as_ref().map(|v| match v {
                MachineRunStatus::Idle => Severity::Normal,
                MachineRunStatus::Running => Severity::Warning,
            }),
        )
        .draw(target, draw_type)?;

        let cursor = cursor + Point::new(0, 5);

        // Coolant flow temperature
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Cool Flw",
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

        // Coolant return temperature
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Cool Rtn",
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

        let cursor = cursor + Point::new(0, 5);

        // Coolant resevoir top temperature
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

        // Coolant resevoir bottom temperature
        Measurement::new(
            cursor,
            value_offset,
            "Res Bott",
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

        Ok(())
    }
}
