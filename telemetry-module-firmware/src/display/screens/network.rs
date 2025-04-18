use crate::{
    display::{
        drawables::{
            info_background::INFO_PANE_REGION, measurement::Measurement, subtitle::Subtitle,
        },
        DrawType, DrawTypeDrawable,
    },
    network::WIFI_SSID,
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

        let cursor = Subtitle::new(cursor, "Local Network").draw(target, draw_type)?;

        // WiFi SSID
        let cursor = Measurement::new(cursor, value_offset, "SSID", Some(WIFI_SSID))
            .draw(target, draw_type)?;

        // Network connection status
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "State",
            None,
            // Some(match self.state.ipv4_config {
            //     Some(_) => "Connected",
            //     None => "Disconnected",
            // }),
        )
        .draw(target, draw_type)?;

        // Our IP address
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "IP",
            None,
            // self.state
            //     .ipv4_config
            //     .as_ref()
            //     .map(|config| {
            //         let mut s = heapless::String::<16>::new();
            //         s.write_fmt(format_args!("{}", config.address.address()))
            //             .unwrap();
            //         s
            //     })
            //     .as_deref(),
        )
        .draw(target, draw_type)?;

        // Gateway IP address
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Gtwy",
            None,
            // self.state
            //     .ipv4_config
            //     .as_ref()
            //     .map(|config| {
            //         let mut s = heapless::String::<16>::new();
            //         s.write_fmt(format_args!("{}", config.gateway.unwrap()))
            //             .unwrap();
            //         s
            //     })
            //     .as_deref(),
        )
        .draw(target, draw_type)?;

        // TODO
        let cursor = cursor + Point::new(0, 5);

        Ok(())
    }
}
