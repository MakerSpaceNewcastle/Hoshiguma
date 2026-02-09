mod app;
mod backlight;
mod touchscreen;

use self::touchscreen::SoftButton;
pub(crate) use self::{
    app::set_status_screen_info,
    backlight::{BACKLIGHT_MODE, BACKLIGHT_WAKE},
};
use crate::ui::backlight::BACKLIGHT_ON;
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, watch::Watch};
use embassy_time::Timer;
use embedded_graphics::prelude::Point;
use hoshiguma_api::hmi::Screen;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, fonts};
use peek_o_display_bsp::{
    display::Display,
    embassy_rp::gpio::{Input, Output},
    touch::Touch,
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn init(
    spawner: Spawner,
    display: Display,
    touch: Touch,
    touch_irq: Input<'static>,
    backlight: Output<'static>,
) {
    spawner.spawn(task(display).unwrap());
    spawner.spawn(touchscreen::task(touch, touch_irq).unwrap());
    spawner.spawn(backlight::task(backlight).unwrap());
}

static UI_TOUCH_POINT: Watch<CriticalSectionRawMutex, (Point, Option<SoftButton>), 1> =
    Watch::new();

static CHANGE_SCREEN_CH: Channel<CriticalSectionRawMutex, Screen, 1> = Channel::new();

pub(crate) async fn change_screen(screen: Screen) {
    CHANGE_SCREEN_CH.send(screen).await;
}

#[embassy_executor::task]
async fn task(mut display: Display) {
    let config = EmbeddedBackendConfig {
        font_regular: fonts::MONO_9X15,
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();

    // Show the soft button touch boxes
    touchscreen::draw_soft_button_touch_boxes(terminal.backend_mut().display_mut());
    Timer::after_millis(500).await;
    terminal.clear().unwrap();

    // Show the splash screen
    terminal
        .draw(|f| {
            draw_splash_screen(f);
        })
        .unwrap();
    Timer::after_secs(1).await;

    let mut app = app::App::new();
    let mut touch_point_rx = UI_TOUCH_POINT.receiver().unwrap();
    let change_screen_rx = CHANGE_SCREEN_CH.receiver();
    let mut redraw = true;

    loop {
        if redraw {
            terminal
                .draw(|f| {
                    app.render(f);
                })
                .unwrap();
        }

        match select3(
            touch_point_rx.changed(),
            change_screen_rx.receive(),
            app.await_data(),
        )
        .await
        {
            Either3::First(point) => {
                // If the backlight is off, just wake it without handling the touch event. This prevents accidental touches from doing things when backlight is off and screen therefore unreadible.
                let backlight_on = BACKLIGHT_ON.try_get().unwrap_or(false);
                BACKLIGHT_WAKE.send(()).await;
                redraw = if !backlight_on {
                    true
                } else {
                    app.handle_touch(point)
                };
            }
            Either3::Second(screen) => {
                app.change_screen(screen);
                redraw = true;
            }
            Either3::Third(r) => {
                redraw = r;
            }
        };
    }
}

fn draw_splash_screen(f: &mut ratatui::Frame) {
    let area = f.area();

    let block = Block::default().borders(Borders::ALL);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .split(block.inner(area));

    let title_text = Paragraph::new("Hoshiguma HMI").centered();
    let git_rev_text = Paragraph::new(git_version::git_version!())
        .centered()
        .yellow();

    f.render_widget(block, area);
    f.render_widget(title_text, vertical_chunks[1]);
    f.render_widget(git_rev_text, vertical_chunks[2]);
}
