use crate::display::{
    drawables::{
        measurement::{UNKNOWN_COLOUR, UNKNOWN_TEXT},
        screen::{INFO_PANE_BACKGROUND_COLOUR, INFO_PANE_REGION},
        text::GenericText,
    },
    state::DisplayDataState,
    DrawType, DrawTypeDrawable, LIGHT_TEXT_COLOUR,
};
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, WebColors},
    primitives::PrimitiveStyleBuilder,
    text::{renderer::CharacterStyle, Alignment, Text},
    Drawable,
};
use hoshiguma_telemetry_protocol::payload::process::{Monitor, MonitorState};

pub(super) struct AlarmList<'a> {
    state: &'a DisplayDataState,
}

impl<'a> AlarmList<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for AlarmList<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, _draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Always redraw the background
        let background_style = PrimitiveStyleBuilder::new()
            .fill_color(INFO_PANE_BACKGROUND_COLOUR)
            .build();
        INFO_PANE_REGION
            .into_styled(background_style)
            .draw(target)?;

        if let Some(alarms) = &self.state.alarms {
            if alarms.alarms.is_empty() {
                Text::with_alignment(
                    "No Alarms\nyey~",
                    INFO_PANE_REGION.center(),
                    MonoTextStyle::new(&FONT_6X10, LIGHT_TEXT_COLOUR),
                    Alignment::Center,
                )
                .draw(target)?;
            } else {
                let cursor = Point::new(
                    INFO_PANE_REGION.top_left.x + 2,
                    INFO_PANE_REGION.top_left.y + 11,
                );

                let mut cursor = {
                    let mut s = heapless::String::<16>::new();
                    s.write_fmt(format_args!("{} alarm(s)!", alarms.alarms.len()))
                        .unwrap();
                    GenericText::new(cursor, &s).draw(target, &DrawType::Full)?
                };

                for (i, alarm) in alarms.alarms.iter().enumerate() {
                    // Only so many items can fit on the display
                    if i > 6 {
                        break;
                    }

                    let name = match alarm.monitor {
                        Monitor::LogicPowerSupplyNotPresent => "Logic Power Inop.",
                        Monitor::ChassisIntrusion => "Chassis Intrusion",
                        Monitor::CoolantResevoirLevelSensorFault => "Cool. Lvl. S. Fault",
                        Monitor::CoolantResevoirLevel => "Coolant Res. Level",
                        Monitor::TemperatureSensorFault => "Temp. Sensor Fault",
                        Monitor::CoolantFlowTemperature => "Coolant Flow Hot",
                        Monitor::CoolantResevoirTemperature => "Coolant Res. Hot",
                    };

                    let mut text = GenericText::new(cursor, name);
                    text.style.set_text_color(Some(match alarm.state {
                        MonitorState::Normal => Rgb565::CSS_GREEN,
                        MonitorState::Warn => Rgb565::CSS_YELLOW,
                        MonitorState::Critical => Rgb565::CSS_RED,
                    }));
                    cursor = text.draw(target, &DrawType::Full)?;
                }
            }
        } else {
            Text::with_alignment(
                UNKNOWN_TEXT,
                INFO_PANE_REGION.center(),
                MonoTextStyle::new(&FONT_6X10, UNKNOWN_COLOUR),
                Alignment::Center,
            )
            .draw(target)?;
        }

        Ok(())
    }
}