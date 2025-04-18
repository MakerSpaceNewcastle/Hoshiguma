use crate::{
    display::{
        drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
        DrawType, DrawTypeDrawable,
    },
    network::{IP_CONFIG, WIFI_SSID},
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Network {}

impl DrawTypeDrawable for Network {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 35;
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // WiFi SSID
        let cursor = Measurement::new(cursor, value_offset, "SSID", Some(WIFI_SSID))
            .draw(target, draw_type)?;

        // Our IP address
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "IP",
            IP_CONFIG
                .try_get()
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
            "Gtwy",
            IP_CONFIG
                .try_get()
                .map(|config| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", config.gateway.unwrap()))
                        .unwrap();
                    s
                })
                .as_deref(),
        )
        .draw(target, draw_type)?;

        // DNS IP address 1
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "DNS1",
            IP_CONFIG
                .try_get()
                .map(|config| {
                    config.dns_servers.get(0).map(|addr| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{}", addr)).unwrap();
                        s
                    })
                })
                .flatten()
                .as_deref(),
        )
        .draw(target, draw_type)?;

        // DNS IP address 2
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "DNS2",
            IP_CONFIG
                .try_get()
                .map(|config| {
                    config.dns_servers.get(1).map(|addr| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{}", addr)).unwrap();
                        s
                    })
                })
                .flatten()
                .as_deref(),
        )
        .draw(target, draw_type)?;

        // DNS IP address 3
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "DNS3",
            IP_CONFIG
                .try_get()
                .map(|config| {
                    config.dns_servers.get(2).map(|addr| {
                        let mut s = heapless::String::<16>::new();
                        s.write_fmt(format_args!("{}", addr)).unwrap();
                        s
                    })
                })
                .flatten()
                .as_deref(),
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
