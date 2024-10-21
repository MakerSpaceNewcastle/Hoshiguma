use crate::io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs};
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static COOLANT_RESEVOIR_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    CoolantResevoirLevelReading,
    2,
> = Watch::new();

#[macro_export]
macro_rules! init_coolant_resevoir_level_sensor {
    ($p:expr) => {{
        // Level shifted IO 4
        let empty = embassy_rp::gpio::Input::new($p.PIN_4, embassy_rp::gpio::Pull::Up);

        // Level shifted IO 5
        let low = embassy_rp::gpio::Input::new($p.PIN_5, embassy_rp::gpio::Pull::Up);

        $crate::devices::coolant_resevoir_level_sensor::CoolantResevoirLevelSensor::new([
            empty, low,
        ])
    }};
}

pub(crate) type CoolantResevoirLevelSensor =
    DigitalInputStateChangeDetector<2, CoolantResevoirLevelReading>;

#[derive(Clone, Format)]
pub(crate) struct CoolantResevoirLevelReading(pub(crate) Result<CoolantResevoirLevel, ()>);

#[cfg(feature = "telemetry")]
impl From<&CoolantResevoirLevelReading>
    for hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevelReading
{
    fn from(value: &CoolantResevoirLevelReading) -> Self {
        match value.0 {
            Ok(CoolantResevoirLevel::Full) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Full)
            }
            Ok(CoolantResevoirLevel::Low) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Low)
            }
            Ok(CoolantResevoirLevel::Empty) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Empty)
            }
            Err(_) => Err(()),
        }
    }
}

#[derive(Clone, Format)]
pub(crate) enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}

impl StateFromDigitalInputs<2> for CoolantResevoirLevelReading {
    fn from_inputs(inputs: [Level; 2]) -> Self {
        Self(match (inputs[0], inputs[1]) {
            (Level::Low, Level::Low) => Ok(CoolantResevoirLevel::Empty),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(CoolantResevoirLevel::Low),
            (Level::High, Level::High) => Ok(CoolantResevoirLevel::Full),
        })
    }
}
