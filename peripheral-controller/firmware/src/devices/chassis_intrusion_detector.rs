use crate::io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs};
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static CHASSIS_INTRUSION_CHANGED: Watch<CriticalSectionRawMutex, ChassisIntrusion, 1> =
    Watch::new();

#[macro_export]
macro_rules! init_chassis_intrusion_detector {
    ($p:expr) => {{
        // Isolated input 6
        let input = embassy_rp::gpio::Input::new($p.PIN_9, embassy_rp::gpio::Pull::Down);

        $crate::devices::chassis_intrusion_detector::ChassisIntrusionDetector::new([input])
    }};
}

pub(crate) type ChassisIntrusionDetector = DigitalInputStateChangeDetector<1, ChassisIntrusion>;

#[derive(Clone, Format)]
pub(crate) enum ChassisIntrusion {
    Normal,
    Intruded,
}

#[cfg(feature = "telemetry")]
impl From<&ChassisIntrusion>
    for hoshiguma_telemetry_protocol::payload::observation::ChassisIntrusion
{
    fn from(value: &ChassisIntrusion) -> Self {
        match value {
            ChassisIntrusion::Normal => Self::Normal,
            ChassisIntrusion::Intruded => Self::Intruded,
        }
    }
}

impl StateFromDigitalInputs<1> for ChassisIntrusion {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Intruded,
            Level::High => Self::Normal,
        }
    }
}
