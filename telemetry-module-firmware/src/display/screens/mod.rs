mod auto_summary;
mod device;
mod inputs;
mod monitors;
mod network;
mod outputs;
mod summary;
mod temperatures;

use super::{state::DisplayDataState, DrawType};
use crate::display::DrawTypeDrawable;
use defmt::{debug, info, Format};
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

pub(super) trait DrawableScreen {
    type Color;
    type Output;

    fn draw<D>(
        &self,
        target: &mut D,
        draw_type: &DrawType,
        state: &DisplayDataState,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
}

#[derive(Clone, Format)]
pub(super) enum Screen {
    AutoSummary,
    Summary,
    Monitors,
    Temperatures,
    Inputs,
    Outputs,
    Network,
    Device,
}

impl Screen {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Screen::AutoSummary => "Home",
            Screen::Summary => "Summary",
            Screen::Monitors => "Monitors",
            Screen::Temperatures => "Temperatures",
            Screen::Inputs => "Inputs",
            Screen::Outputs => "Outputs",
            Screen::Network => "Network",
            Screen::Device => "Device",
        }
    }
}

pub(super) const SCREENS: [Screen; 8] = [
    Screen::AutoSummary,
    Screen::Summary,
    Screen::Monitors,
    Screen::Temperatures,
    Screen::Inputs,
    Screen::Outputs,
    Screen::Network,
    Screen::Device,
];

#[derive(Default, Format)]
pub(super) struct ScreenSelector {
    selected_idx: usize,
}

impl ScreenSelector {
    pub(super) fn num_screens() -> usize {
        SCREENS.len()
    }

    pub(super) fn select_next(&mut self) {
        self.selected_idx += 1;

        if self.selected_idx >= SCREENS.len() {
            self.selected_idx = 0;
        }

        info!("Selected screen is now {}", self.current_screen());
    }

    pub(super) fn current_screen(&self) -> &'static Screen {
        &SCREENS[self.selected_idx]
    }

    pub(super) fn current_screen_number(&self) -> usize {
        self.selected_idx + 1
    }
}

impl DrawableScreen for ScreenSelector {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(
        &self,
        target: &mut D,
        draw_type: &DrawType,
        state: &DisplayDataState,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let screen = self.current_screen();
        debug!("Drawing screen {}", screen);

        match screen {
            Screen::AutoSummary => {
                self::auto_summary::AutoSummary::new(state).draw(target, draw_type)
            }
            Screen::Summary => self::summary::Summary::new(state).draw(target, draw_type),
            Screen::Monitors => self::monitors::Monitors::new(state).draw(target, draw_type),
            Screen::Temperatures => {
                self::temperatures::Temperatures::new(state).draw(target, draw_type)
            }
            Screen::Inputs => self::inputs::Inputs::new(state).draw(target, draw_type),
            Screen::Outputs => self::outputs::Outputs::new(state).draw(target, draw_type),
            Screen::Network => self::network::Network::new(state).draw(target, draw_type),
            Screen::Device => self::device::Device::new(state).draw(target, draw_type),
        }
    }
}
