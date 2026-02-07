use crate::{
    display::{DrawType, DrawTypeDrawable, drawables::measurement::Measurement},
    network::LINK_STATE,
    self_telemetry::{
        DATA_POINTS_ACCEPTED, DATA_POINTS_DISCARDED, RPC_REQUEST_DATAPOINT, STRING_REGISTRY_SIZE,
        TELEGRAF_SUBMIT_FAIL, TELEGRAF_SUBMIT_SUCCESS,
    },
};
use core::{fmt::Write, sync::atomic::Ordering};
use embassy_time::Instant;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(crate) struct Diagnostics {}

impl DrawTypeDrawable for Diagnostics {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 105;
        let cursor = Point::new(
            target.bounding_box().top_left.x + 2,
            target.bounding_box().top_left.y + 12,
        );

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Git Revision",
            Some(git_version::git_version!()),
        )
        .draw(target, draw_type)?;

        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{}s", Instant::now().as_secs()))
            .unwrap();
        let cursor =
            Measurement::new(cursor, value_offset, "Uptime", Some(&s)).draw(target, draw_type)?;

        let boot_reason = embassy_rp::pac::WATCHDOG.reason().read();
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Boot Reason",
            Some(if boot_reason.force() {
                "Forced Reset"
            } else if boot_reason.timer() {
                "Watchdog Timeout"
            } else {
                "Normal"
            }),
        )
        .draw(target, draw_type)?;

        let link_state = LINK_STATE.lock(|v| v.borrow().clone());

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "IP",
            link_state
                .dhcp4_config
                .clone()
                .map(|config| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", config.address.address()))
                        .unwrap();
                    s
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Gateway",
            link_state
                .dhcp4_config
                .clone()
                .map(|config| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", config.gateway.unwrap()))
                        .unwrap();
                    s
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "DNS 1",
            link_state
                .dhcp4_config
                .clone()
                .and_then(|config| {
                    config.dns_servers.first().map(|addr| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{addr}")).unwrap();
                        s
                    })
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Conn. Age",
            Some(
                {
                    let age = link_state.age().as_secs();
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{age}s")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Strings",
            Some(
                {
                    let count = STRING_REGISTRY_SIZE.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Strings Age",
            Some(
                {
                    let age =
                        crate::self_telemetry::string_registry_last_modification_age_ms() / 1000;
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{age}s")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#RPC data pt.",
            Some(
                {
                    let count = RPC_REQUEST_DATAPOINT.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Buf Ins",
            Some(
                {
                    let count = DATA_POINTS_ACCEPTED.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Buf",
            Some(
                {
                    let count = DATA_POINTS_DISCARDED.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Msgs Tx",
            Some(
                {
                    let count = TELEGRAF_SUBMIT_SUCCESS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        let _cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Net",
            Some(
                {
                    let count = TELEGRAF_SUBMIT_FAIL.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
