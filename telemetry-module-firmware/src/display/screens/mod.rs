mod home;
mod network;
mod telemetry;
mod telemetry_module;
mod time;

use super::DrawType;
use crate::display::DrawTypeDrawable;
use defmt::{debug, info, Format};
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

#[derive(Clone, Format)]
pub(super) enum Screen {
    Home,
    Telemetry,
    Time,
    Network,
    Module,
}

impl Screen {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Screen::Home => "Home",
            Screen::Telemetry => "Telemetry",
            Screen::Time => "Time",
            Screen::Network => "Network",
            Screen::Module => "Telem. Module",
        }
    }
}

pub(super) const SCREENS: [Screen; 5] = [
    Screen::Home,
    Screen::Telemetry,
    Screen::Time,
    Screen::Network,
    Screen::Module,
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

impl DrawTypeDrawable for ScreenSelector {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let screen = self.current_screen();
        debug!("Drawing screen {}", screen);

        match screen {
            Screen::Home => self::home::Home {}.draw(target, draw_type),
            Screen::Telemetry => self::telemetry::Telemetry {}.draw(target, draw_type),
            Screen::Time => self::time::Time {}.draw(target, draw_type),
            Screen::Network => self::network::Network {}.draw(target, draw_type),
            Screen::Module => self::telemetry_module::TelemetryModule {}.draw(target, draw_type),
        }
    }
}
