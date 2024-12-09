use crate::{
    display::{
        drawables::{
            measurement::{Measurement, Severity},
            screen::INFO_PANE_REGION,
            subtitle::Subtitle,
        },
        state::DisplayDataState,
        DrawType, DrawTypeDrawable,
    },
    wifi::MQTT_BROKER_IP,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Network<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Network<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Network<'_> {
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

        // Network connection status
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "State",
            Some(match self.state.ipv4_config {
                Some(_) => "Connected",
                None => "Disconnected",
            }),
            Some(match self.state.ipv4_config {
                Some(_) => Severity::Normal,
                None => Severity::Critical,
            }),
        )
        .draw(target, draw_type)?;

        // Our IP address
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "IP",
            self.state
                .ipv4_config
                .as_ref()
                .map(|config| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", config.address.address()))
                        .unwrap();
                    s
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        // Gateway IP address
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "Gtwy",
            self.state
                .ipv4_config
                .as_ref()
                .map(|config| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", config.gateway.unwrap()))
                        .unwrap();
                    s
                })
                .as_deref(),
            None,
        )
        .draw(target, draw_type)?;

        let cursor = cursor + Point::new(0, 5);

        let cursor = Subtitle::new(cursor, "MQTT").draw(target, draw_type)?;

        // MQTT connection status
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "State",
            Some(match self.state.mqtt_broker_connected {
                true => "Connected",
                false => "Disconnected",
            }),
            Some(match self.state.mqtt_broker_connected {
                true => Severity::Normal,
                false => Severity::Critical,
            }),
        )
        .draw(target, draw_type)?;

        // MQTT broker IP address
        let mqtt_broker_ip_str = {
            let mut s = heapless::String::<16>::new();
            s.write_fmt(format_args!("{}", MQTT_BROKER_IP)).unwrap();
            s
        };
        Measurement::new(
            cursor,
            value_offset,
            "Brkr",
            Some(&mqtt_broker_ip_str),
            None,
        )
        .draw(target, draw_type)?;

        Ok(())
    }
}
