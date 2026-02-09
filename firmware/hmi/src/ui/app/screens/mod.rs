use crate::ui::SoftButton;
use embedded_graphics::prelude::Point;
use hoshiguma_api::hmi::Screen;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

pub(super) mod hmi_info;
pub(super) mod status;

pub(super) trait ScreenExt {
    fn render(&mut self, f: &mut Frame);
    fn handle_touch(&mut self, event: (Point, Option<SoftButton>)) -> UpdateAction;
    async fn await_data(&mut self) -> UpdateAction;
}

pub(super) enum UpdateAction {
    Nothing,
    Redraw,
    ChangeToScreen(Screen),
}

fn draw_buttons(f: &mut Frame, area: Rect, buttons: [(&str, Style); 3]) {
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(area);

    for (i, (label, style)) in buttons.iter().enumerate() {
        let button_block = Block::default().borders(Borders::ALL).style(*style);
        let button_text = Paragraph::new(*label).centered().block(button_block);
        f.render_widget(button_text, bottom_chunks[i * 2]);
    }
}
