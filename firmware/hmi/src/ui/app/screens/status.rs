use super::UpdateAction;
use crate::ui::{SoftButton, app::screens::draw_buttons};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Watch},
};
use embedded_graphics::prelude::Point;
use hoshiguma_api::{
    DesiredMachinePower, Interlock, MachineRun, Severity,
    hmi::{AccessControlRawInput, Screen, StatusScreenInfo},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

static STATUS_SCREEN_INFO: Watch<CriticalSectionRawMutex, StatusScreenInfo, 1> = Watch::new();

pub(crate) fn set_status_screen_info(info: StatusScreenInfo) {
    STATUS_SCREEN_INFO.sender().send(info);
}

pub(crate) struct StatusScreen {
    info_rx: Receiver<'static, CriticalSectionRawMutex, StatusScreenInfo, 1>,
    info: Option<StatusScreenInfo>,
}

impl StatusScreen {
    pub(crate) fn new() -> Self {
        Self {
            info_rx: STATUS_SCREEN_INFO.receiver().unwrap(),
            info: None,
        }
    }
}

impl super::ScreenExt for StatusScreen {
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        let title = Paragraph::new("Hoshiguma: Main Status")
            .centered()
            .underlined();
        f.render_widget(title, vertical_layout[0]);

        let status_layout_1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(vertical_layout[2]);

        f.render_widget(
            render_access_control(self.info.as_ref()),
            status_layout_1[0],
        );

        f.render_widget(render_machine_power(self.info.as_ref()), status_layout_1[1]);

        let status_layout_2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(vertical_layout[3]);

        f.render_widget(render_interlock(self.info.as_ref()), status_layout_2[0]);

        f.render_widget(render_status(self.info.as_ref()), status_layout_2[1]);

        f.render_widget(render_messages(self.info.as_ref()), vertical_layout[4]);

        draw_buttons(
            f,
            vertical_layout[5],
            [
                ("PAGE", Style::default().white()),
                ("", Style::default().gray()),
                ("", Style::default().gray()),
            ],
        );
    }

    fn handle_touch(&mut self, event: (Point, Option<SoftButton>)) -> UpdateAction {
        match event.1 {
            Some(SoftButton::A) => UpdateAction::ChangeToScreen(Screen::HmiInfo),
            _ => UpdateAction::Nothing,
        }
    }

    async fn await_data(&mut self) -> UpdateAction {
        let new_info = self.info_rx.changed().await;
        self.info = Some(new_info);
        UpdateAction::Redraw
    }
}

fn var_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .white()
        .on_black()
}

fn render_access_control<'a>(info: Option<&'a StatusScreenInfo>) -> Paragraph<'a> {
    let text = match info {
        Some(info) => match info.access_control {
            AccessControlRawInput::Idle => "".black(),
            AccessControlRawInput::Denied => "Denied".black().on_red(),
            AccessControlRawInput::Granted => "Granted".green(),
        },
        None => "NO DATA".on_magenta(),
    };

    Paragraph::new(text)
        .centered()
        .block(var_block("Acc. Ctrl."))
}

fn render_machine_power<'a>(info: Option<&'a StatusScreenInfo>) -> Paragraph<'a> {
    let text = match info {
        Some(info) => match info.machine_power {
            DesiredMachinePower::Off => "Off".white(),
            DesiredMachinePower::On => "On".yellow(),
        },
        None => "NO DATA".on_magenta(),
    };

    Paragraph::new(text).centered().block(var_block("Power"))
}

fn render_interlock<'a>(info: Option<&'a StatusScreenInfo>) -> Paragraph<'a> {
    let text = match info {
        Some(info) => match info.interlock {
            Interlock::OperationPermitted => "OK".green(),
            Interlock::OperationPermittedUntilIdle => "OK w. Run".yellow(),
            Interlock::OperationDenied => "Denied".red(),
            Interlock::MachineProtected => "Protect".black().on_red(),
        },
        None => "NO DATA".on_magenta(),
    };

    Paragraph::new(text)
        .centered()
        .block(var_block("Interlock"))
}

fn render_status<'a>(info: Option<&'a StatusScreenInfo>) -> Paragraph<'a> {
    let text = match info {
        Some(info) => match info.running {
            MachineRun::Idle => "Idle".white(),
            MachineRun::Running => "Running".yellow(),
        },
        None => "NO DATA".on_magenta(),
    };

    Paragraph::new(text).centered().block(var_block("Status"))
}

fn render_messages<'a>(info: Option<&'a StatusScreenInfo>) -> Paragraph<'a> {
    let t = match info {
        Some(info) => {
            let mut t = Text::default();

            for l in &info.messages {
                match l.severity {
                    Severity::Information => t.push_line(l.text.blue()),
                    Severity::Normal => t.push_line(l.text.white()),
                    Severity::Warning => t.push_line(l.text.yellow()),
                    Severity::Critical => t.push_line(l.text.red()),
                    Severity::Fatal => t.push_line(l.text.black().on_red()),
                };
            }

            t
        }
        None => Text::from("NO DATA".on_magenta()),
    };

    Paragraph::new(t)
        .left_aligned()
        .block(var_block("Messages"))
}
