use crate::{
    buttons::{BUTTON_EVENTS, Button, ButtonEvent},
    display::{DISPLAY_TEXT, DisplayText},
};
use chrono::{Datelike, Timelike};
use core::fmt::Write;
use defmt::{debug, error, info};
use embassy_futures::select::{Either, select};
use embassy_net::Stack;
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Instant, Ticker};

pub(crate) struct Context {
    pub(crate) network_external: Stack<'static>,
}

#[embassy_executor::task]
pub(crate) async fn task(ctx: Context) -> ! {
    let text_tx = DISPLAY_TEXT.sender();
    let mut button_rx = BUTTON_EVENTS.subscriber().unwrap();

    let mut tick = Ticker::every(Duration::from_secs(1));

    let mut page = Page::DeviceInfo;

    const TIMEOUT: Duration = Duration::from_secs(10);
    let mut display_timeout = Some(Instant::now() + TIMEOUT);

    loop {
        match select(button_rx.next_message(), tick.next()).await {
            Either::First(WaitResult::Message(ButtonEvent::Pressed(Button::One))) => {
                // Wake the display
                display_timeout = Some(Instant::now() + TIMEOUT);

                // Render the display now
                tick.reset_at(Instant::now() - Duration::from_secs(1));
            }
            Either::First(WaitResult::Message(ButtonEvent::Pressed(Button::Two))) => {
                // Wake the display and cycle to the next page
                display_timeout = Some(Instant::now() + TIMEOUT);
                page = page.next();

                // Render the display now
                tick.reset_at(Instant::now() - Duration::from_secs(1));
            }
            Either::First(WaitResult::Message(_)) => {
                // Do nothing
            }
            Either::First(WaitResult::Lagged(n)) => {
                error!("{} button events were dropped", n);
            }
            Either::Second(_) => {
                // Check if the display timeout has expired
                if let Some(timeout) = display_timeout {
                    if Instant::now() >= timeout {
                        info!("Display timeout");
                        display_timeout = None;
                        text_tx.send("".try_into().unwrap());
                    }
                }

                if display_timeout.is_some() {
                    debug!("Updating display");
                    let text = page.string_info(&ctx);
                    text_tx.send(text);
                }
            }
        }
    }
}

enum Page {
    DeviceInfo,
    DateTime,
    ExternalNetwork,
}

impl Page {
    fn next(self) -> Self {
        match self {
            Page::DeviceInfo => Page::DateTime,
            Page::DateTime => Page::ExternalNetwork,
            Page::ExternalNetwork => Page::DeviceInfo,
        }
    }

    fn string_info(&self, ctx: &Context) -> DisplayText {
        let mut text = DisplayText::new();

        match self {
            Page::DeviceInfo => {
                text.write_str("Hoshiguma\nTelemetry Bridge\n\n").unwrap();
                text.write_str(git_version::git_version!()).unwrap();
                text.write_char('\n').unwrap();
            }
            Page::DateTime => {
                let time = crate::wall_time::now();

                text.write_str("Date: ").unwrap();
                match time {
                    Some(time) => text.write_fmt(format_args!(
                        "{}-{:02}-{:02}\n",
                        time.year(),
                        time.month(),
                        time.day()
                    )),
                    None => text.write_str("unknown\n"),
                }
                .unwrap();

                text.write_str("Time: ").unwrap();
                match time {
                    Some(time) => text.write_fmt(format_args!(
                        "{:02}:{:02}:{:02}\n",
                        time.hour(),
                        time.minute(),
                        time.second()
                    )),
                    None => text.write_str("unknown\n"),
                }
                .unwrap();
            }
            Page::ExternalNetwork => {
                text.write_str("External Network\n").unwrap();

                text.write_str("Link  : ").unwrap();
                text.write_str(match ctx.network_external.is_link_up() {
                    true => "up",
                    false => "down",
                })
                .unwrap();
                text.write_char('\n').unwrap();

                text.write_str("Config: ").unwrap();
                text.write_str(match ctx.network_external.is_config_up() {
                    true => "up",
                    false => "down",
                })
                .unwrap();
                text.write_char('\n').unwrap();

                text.write_str("IP: ").unwrap();
                match ctx.network_external.config_v4() {
                    Some(config) => text.write_fmt(format_args!("{}", config.address)),
                    None => text.write_str("none"),
                }
                .unwrap();
                text.write_char('\n').unwrap();
            }
        }

        text
    }
}
