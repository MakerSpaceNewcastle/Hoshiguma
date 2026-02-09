mod screens;
pub(crate) use screens::status::set_status_screen_info;

use crate::ui::{
    SoftButton,
    app::screens::{ScreenExt, UpdateAction, hmi_info::HmiInfoScreen, status::StatusScreen},
};
use defmt::debug;
use embedded_graphics::prelude::Point;
use hoshiguma_api::hmi::Screen;
use ratatui::Frame;

pub(super) struct App {
    active_screen: Screen,

    status_screen: StatusScreen,
    hmi_info_screen: HmiInfoScreen,
}

impl App {
    pub(super) fn new() -> Self {
        Self {
            active_screen: Screen::Status,

            status_screen: StatusScreen::new(),
            hmi_info_screen: HmiInfoScreen::new(),
        }
    }

    pub(super) fn change_screen(&mut self, screen: Screen) {
        self.active_screen = screen;
    }

    pub(super) fn render(&mut self, f: &mut Frame) {
        match self.active_screen {
            Screen::Status => self.status_screen.render(f),
            Screen::HmiInfo => self.hmi_info_screen.render(f),
        }
    }

    pub(super) fn handle_touch(&mut self, event: (Point, Option<SoftButton>)) -> bool {
        debug!("Touched {}", event.1);
        let action = match self.active_screen {
            Screen::Status => self.status_screen.handle_touch(event),
            Screen::HmiInfo => self.hmi_info_screen.handle_touch(event),
        };

        match action {
            UpdateAction::Nothing => false,
            UpdateAction::Redraw => true,
            UpdateAction::ChangeToScreen(screen) => {
                self.active_screen = screen;
                true
            }
        }
    }

    pub(super) async fn await_data(&mut self) -> bool {
        let action = match self.active_screen {
            Screen::Status => self.status_screen.await_data().await,
            Screen::HmiInfo => self.hmi_info_screen.await_data().await,
        };

        match action {
            UpdateAction::Nothing => false,
            UpdateAction::Redraw => true,
            UpdateAction::ChangeToScreen(screen) => {
                self.active_screen = screen;
                true
            }
        }
    }
}
