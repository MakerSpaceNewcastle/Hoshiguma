use crate::display::{
    drawables::{
        info_background::{InfoPaneBackground, INFO_PANE_REGION},
        measurement::{UNKNOWN_COLOUR, UNKNOWN_TEXT},
        text::GenericText,
    },
    state::DisplayDataState,
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, WebColors},
    text::{renderer::CharacterStyle, Alignment, Text},
    Drawable,
};
use hoshiguma_protocol::{peripheral_controller::types::MonitorKind, types::Severity};

pub(super) struct Monitors<'a> {
    state: &'a DisplayDataState,
}

impl<'a> Monitors<'a> {
    pub(super) fn new(state: &'a DisplayDataState) -> Self {
        Self { state }
    }
}

impl DrawTypeDrawable for Monitors<'_> {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, _draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Always redraw the background
        InfoPaneBackground::default().draw(target)?;

        if let Some(monitors) = &self.state.monitors {
            let mut cursor = Point::new(
                INFO_PANE_REGION.top_left.x + 2,
                INFO_PANE_REGION.top_left.y + 11,
            );

            for (i, (monitor, severity)) in monitors.iter().enumerate() {
                // Only so many items can fit on the display
                if i > 6 {
                    break;
                }

                let name = match monitor {
                    MonitorKind::LogicPowerSupplyNotPresent => "Logic Power Inop.",
                    MonitorKind::ChassisIntrusion => "Chassis Intrusion",
                    MonitorKind::CoolantResevoirLevelSensorFault => "Cool. Lvl. S. Fault",
                    MonitorKind::CoolantResevoirLevel => "Coolant Res. Level",
                    MonitorKind::TemperatureSensorFault => "Temp. Sensor Fault",
                    MonitorKind::CoolantFlowTemperature => "Coolant Flow Hot",
                    MonitorKind::CoolantResevoirTemperature => "Coolant Res. Hot",
                };

                let mut text = GenericText::new(cursor, name);
                text.style.set_text_color(Some(match severity {
                    Severity::Normal => Rgb565::CSS_GREEN,
                    Severity::Warn => Rgb565::CSS_YELLOW,
                    Severity::Critical => Rgb565::CSS_RED,
                }));
                cursor = text.draw(target, &DrawType::Full)?;
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
