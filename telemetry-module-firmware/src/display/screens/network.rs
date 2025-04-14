use crate::{
    display::{
        drawables::{info_background::INFO_PANE_REGION, measurement::Measurement},
        DrawType, DrawTypeDrawable,
    },
    network::{DHCP_CONFIG, WIFI_SSID},
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
        let mut cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        // WiFi SSID
        cursor = Measurement::new(cursor, value_offset, "SSID", Some(WIFI_SSID))
            .draw(target, draw_type)?;

        let dhcp_config = DHCP_CONFIG.lock(|v| v.borrow().clone());

        // Our IP address
        cursor = Measurement::new(
            cursor,
            value_offset,
            "IP",
            dhcp_config
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
        cursor = Measurement::new(
            cursor,
            value_offset,
            "Gtwy",
            dhcp_config
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
        for (name, idx) in [("DNS1", 0), ("DNS2", 1), ("DNS3", 2)] {
            cursor = Measurement::new(
                cursor,
                value_offset,
                name,
                dhcp_config
                    .clone()
                    .and_then(|config| {
                        config.dns_servers.get(idx).map(|addr| {
                            let mut s = heapless::String::<16>::new();
                            s.write_fmt(format_args!("{}", addr)).unwrap();
                            s
                        })
                    })
                    .as_deref(),
            )
            .draw(target, draw_type)?;
        }

        Ok(())
    }
}
