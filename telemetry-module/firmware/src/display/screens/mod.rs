mod alert_list;
mod device;
mod inputs;
mod monitors;
mod network;
mod outputs;
mod summary;
mod temperatures;

use crate::display::DrawTypeDrawable;

use super::{state::DisplayDataState, DrawType};
use defmt::{debug, error, info, Format};
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

#[derive(Clone, Format)]
pub(super) enum Screen {
    Summary,
    AlertList,
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
            Screen::Summary => "Summary",
            Screen::AlertList => "Alerts",
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
    Screen::Summary,
    Screen::AlertList,
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

    pub(super) fn draw<D>(&self, target: &mut D, draw_type: &DrawType, state: &DisplayDataState)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let screen = self.current_screen();
        debug!("Drawing screen {}", screen);

        let result = match screen {
            Screen::Summary => self::summary::Summary::new(state).draw(target, draw_type),
            Screen::AlertList => self::alert_list::AlertList {}.draw(target, draw_type),
            Screen::Monitors => self::monitors::Monitors {}.draw(target, draw_type),
            Screen::Temperatures => {
                self::temperatures::Temperatures::new(state).draw(target, draw_type)
            }
            Screen::Inputs => self::inputs::Inputs::new(state).draw(target, draw_type),
            Screen::Outputs => self::outputs::Outputs::new(state).draw(target, draw_type),
            Screen::Network => self::network::Network::new(state).draw(target, draw_type),
            Screen::Device => self::device::Device::new(state).draw(target, draw_type),
        };

        if result.is_err() {
            error!("Failed to draw info pane screen");
        }
    }
}
