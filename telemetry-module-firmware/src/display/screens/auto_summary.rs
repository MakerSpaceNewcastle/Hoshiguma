use crate::display::{
    drawables::info_background::InfoPaneBackground, screens::Screen, state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Drawable},
};

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
        let screen_to_draw = if let Some(alarms) = &self.state.alarms {
            if alarms.alarms.is_empty() {
                Screen::Summary
            } else {
                Screen::AlarmList
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
            Screen::AlarmList => {
                super::alarm_list::AlarmList::new(self.state).draw(target, &DrawType::Full)
            }
            _ => unreachable!("nothing should select a screen other than summary or alarm list"),
        }
    }
}
