use crate::display::{
    drawables::{
        info_pane_background::REGION,
        measurement::{Measurement, Severity},
        subtitle::Subtitle,
    },
    state::DisplayDataState,
};
use core::fmt::Write;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    Drawable,
};

pub(super) struct Network<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Network<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl Drawable for Network<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let value_offset = 35;
        let cursor = Point::new(REGION.top_left.x + 2, REGION.top_left.y + 11);

        let cursor = Subtitle::new(cursor, "Local Network").draw(target)?;

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
        .draw(target)?;

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
        .draw(target)?;

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
        .draw(target)?;

        let cursor = cursor + Point::new(0, 5);

        let cursor = Subtitle::new(cursor, "MQTT").draw(target)?;

        // MQTT connection status
        let cursor = Measurement::new(
            cursor,
            value_offset,
            "State",
            Some(match self.state.mqtt_broker_address {
                Some(_) => "Connected",
                None => "Disconnected",
            }),
            Some(match self.state.mqtt_broker_address {
                Some(_) => Severity::Normal,
                None => Severity::Critical,
            }),
        )
        .draw(target)?;

        // MQTT broker IP address
        Measurement::new(
            cursor,
            value_offset,
            "Brkr",
            self.state
                .mqtt_broker_address
                .map(|addr| {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{}", addr)).unwrap();
                    s
                })
                .as_deref(),
            None,
        )
        .draw(target)?;

        Ok(())
    }
}
