use crate::{
    display::{DrawType, DrawTypeDrawable, drawables::measurement::Measurement},
    network::{
        LINK_STATE,
        telemetry_tx::{
            TELEMETRY_TX_BUFFER_SUBMISSIONS, TELEMETRY_TX_FAIL_BUFFER, TELEMETRY_TX_FAIL_NETWORK,
            TELEMETRY_TX_SUCCESS,
        },
    },
    telemetry::machine::{TELEMETRY_RX_FAIL, TELEMETRY_RX_SUCCESS},
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

        // Git revision of the telemetry module
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Git Revision",
            Some(git_version::git_version!()),
        )
        .draw(target, draw_type)?;

        // Uptime of the telemetry module
        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{}s", Instant::now().as_secs()))
            .unwrap();
        let cursor =
            Measurement::new(cursor, value_offset, "Uptime", Some(&s)).draw(target, draw_type)?;

        // Boot reason of the telemetry module
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

        // Our IP address
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

        // Gateway IP address
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

        // DNS IP addresses
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

        // Connection state age
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

        // Time
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Time",
            crate::network::time::wall_time()
                .map(|time| {
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{}", time.as_secs())).unwrap();
                    s
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        // Seconds since last time sync
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Time Age",
            Some(
                {
                    let age = crate::network::time::time_sync_age().as_secs();
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{age}s")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of events received from peripheral controller
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Events Rx",
            Some(
                {
                    let count = TELEMETRY_RX_SUCCESS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of receive failures
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Rx Fail",
            Some(
                {
                    let count = TELEMETRY_RX_FAIL.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of metrics added to the transmit buffer
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Buf Ins",
            Some(
                {
                    let count = TELEMETRY_TX_BUFFER_SUBMISSIONS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of messages transmitted
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Msgs Tx",
            Some(
                {
                    let count = TELEMETRY_TX_SUCCESS.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of buffer related transmit failures
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Buf",
            Some(
                {
                    let count = TELEMETRY_TX_FAIL_BUFFER.load(Ordering::Relaxed);
                    let mut s = heapless::String::<10>::new();
                    s.write_fmt(format_args!("{count}")).unwrap();
                    s
                }
                .as_ref(),
            ),
        )
        .draw(target, draw_type)?;

        // Number of network related transmit failures
        let _cursor = Measurement::new(
            cursor,
            value_offset,
            "#Tx Fail Net",
            Some(
                {
                    let count = TELEMETRY_TX_FAIL_NETWORK.load(Ordering::Relaxed);
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
