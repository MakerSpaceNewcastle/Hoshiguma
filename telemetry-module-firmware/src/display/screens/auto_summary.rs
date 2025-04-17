use crate::display::{
    drawables::info_background::InfoPaneBackground, screens::Screen, state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Drawable},
};
use hoshiguma_protocol::types::Severity;

/// A screen that shows the alarm list if there are any alarms or the summary screen otherwise.
/// Intended to be a sane default screen to leave the module set to.
pub(super) struct AutoSummary<'a> {
    state: &'a DisplayDataState,
}

impl<'a> AutoSummary<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for AutoSummary<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, _draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let screen_to_draw = if let Some(monitors) = &self.state.monitors {
            if monitors.severity() > Severity::Normal {
                Screen::Monitors
            } else {
                Screen::Summary
            }
        } else {
            Screen::Summary
        };

        match screen_to_draw {
            Screen::Summary => {
                // Always redraw the background
                InfoPaneBackground::default().draw(target)?;

                super::summary::Summary::new(self.state).draw(target, &DrawType::Full)
            }
            Screen::Monitors => {
                super::monitors::Monitors::new(self.state).draw(target, &DrawType::Full)
            }
            _ => unreachable!("nothing should select a screen other than summary or alarm list"),
        }
    }
}
