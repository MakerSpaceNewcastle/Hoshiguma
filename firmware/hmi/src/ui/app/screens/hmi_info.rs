use super::UpdateAction;
use crate::ui::{SoftButton, app::screens::draw_buttons};
use core::fmt::Write;
use defmt::info;
use embassy_time::{Instant, Timer};
use embedded_graphics::prelude::Point;
use heapless::String;
use hoshiguma_api::hmi::Screen;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::Paragraph,
};

pub(crate) struct HmiInfoScreen {}

impl HmiInfoScreen {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl super::ScreenExt for HmiInfoScreen {
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        let title = Paragraph::new("Hoshiguma: HMI Info")
            .centered()
            .underlined();
        f.render_widget(title, vertical_layout[0]);

        let mut s = String::<96>::new();
        s.write_fmt(format_args!(
            "Git revision:\n  {}\nBoot reason:\n  {}\nUptime: {}s",
            git_version::git_version!(),
            crate::boot_reason(),
            Instant::now().as_secs(),
        ))
        .unwrap();
        let text = Paragraph::new(s.as_str()).left_aligned();
        f.render_widget(text, vertical_layout[2]);

        draw_buttons(
            f,
            vertical_layout[3],
            [
                ("PAGE", Style::default().white()),
                ("", Style::default().gray()),
                ("", Style::default().gray()),
            ],
        );
    }

    fn handle_touch(&mut self, event: (Point, Option<SoftButton>)) -> UpdateAction {
        match event.1 {
            Some(SoftButton::A) => UpdateAction::ChangeToScreen(Screen::Status),
            _ => UpdateAction::Nothing,
        }
    }

    async fn await_data(&mut self) -> UpdateAction {
        // Switch back to the status screen after 10 seconds of inactivity, since there is no real reason to stay on this screen for a long time.
        Timer::after_secs(10).await;
        info!("Info screen timed out, returning to status screen");
        UpdateAction::ChangeToScreen(Screen::Status)
    }
}
